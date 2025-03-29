#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "1024"]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResult, pallet_prelude::*,
        traits::{Get, EnsureOrigin, Currency, ReservableCurrency},
    };
    use frame_system::pallet_prelude::*;
    use pallet_timestamp as timestamp;
    use sp_std::vec::Vec;
    use parity_scale_codec::{Encode, Decode};
    use scale_info::TypeInfo;

    /// Log d'ajustement de réputation.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct ReputationLog {
        /// Timestamp (Unix seconds) de l'ajustement.
        pub timestamp: u64,
        /// Variation de réputation (positive ou négative).
        pub delta: i32,
        /// Raison de l'ajustement.
        pub reason: Vec<u8>,
    }

    /// Enregistrement de réputation pour un compte.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct ReputationRecord {
        /// Score de réputation courant.
        pub score: u32,
        /// Historique complet des ajustements.
        pub history: Vec<ReputationLog>,
    }

    /// Types de propositions de gouvernance.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum ProposalType {
        /// Modification du facteur de pénalité.
        UpdatePenaltyFactor,
        // D'autres types de propositions pourront être ajoutés.
    }

    /// Proposition de gouvernance.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct Proposal<AccountId> {
        /// Identifiant unique de la proposition.
        pub id: u32,
        /// Auteur de la proposition.
        pub proposer: AccountId,
        /// Type de proposition.
        pub proposal_type: ProposalType,
        /// Nouvelle valeur proposée.
        pub new_value: u32,
        /// Description détaillée de la proposition.
        pub description: Vec<u8>,
        /// Nombre de votes enregistrés.
        pub vote_count: u32,
        /// Indique si la proposition a été finalisée.
        pub finalized: bool,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config + timestamp::Config {
        /// Type d'événement du runtime.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Score de réputation initial attribué à un nouveau compte.
        #[pallet::constant]
        type InitialReputation: Get<u32>;
        /// Origine autorisée à finaliser les propositions de gouvernance.
        type GovernanceOrigin: EnsureOrigin<Self::RuntimeOrigin>;
        /// Seuil de votes requis pour adopter une proposition.
        #[pallet::constant]
        type ProposalThreshold: Get<u32>;
        /// Monnaie utilisée pour la réservation éventuelle lors des votes.
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
    }

    /// Stockage de la réputation par compte.
    #[pallet::storage]
    #[pallet::getter(fn reputations)]
    pub type Reputations<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, ReputationRecord, OptionQuery>;

    /// Facteur de pénalité global appliqué sur les ajustements négatifs.
    #[pallet::storage]
    #[pallet::getter(fn penalty_factor)]
    pub type PenaltyFactor<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Stockage des propositions de gouvernance.
    #[pallet::storage]
    #[pallet::getter(fn proposals)]
    pub type Proposals<T: Config> =
        StorageMap<_, Blake2_128Concat, u32, Proposal<T::AccountId>, OptionQuery>;

    /// Compteur pour générer des identifiants uniques pour les propositions.
    #[pallet::storage]
    #[pallet::getter(fn proposal_count)]
    pub type ProposalCount<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Stockage des votes sur propositions : (id_proposition, compte) -> bool (vote exprimé).
    #[pallet::storage]
    #[pallet::getter(fn proposal_votes)]
    pub type ProposalVotes<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u32,
        Blake2_128Concat,
        T::AccountId,
        bool,
        OptionQuery
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// La réputation d'un compte a été mise à jour (compte, delta, nouveau score).
        ReputationUpdated(T::AccountId, i32, u32),
        /// Mise à jour du paramètre de gouvernance (nouveau facteur de pénalité).
        GovernanceParameterUpdated(u32),
        /// Création d'une proposition de gouvernance (ID, auteur).
        ProposalCreated(u32, T::AccountId),
        /// Vote enregistré pour une proposition (ID, votant, vote).
        ProposalVoted(u32, T::AccountId, bool),
        /// Finalisation d'une proposition avec adoption de la nouvelle valeur (ID, nouvelle valeur).
        ProposalFinalized(u32, u32),
        /// Ajustement automatique de réputation réalisé (nombre de comptes affectés).
        AutomatedReputationAdjustment(u32),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Enregistrement de réputation introuvable pour ce compte.
        ReputationNotFound,
        /// L'opération conduirait à un underflow de réputation.
        ReputationUnderflow,
        /// Une réputation existe déjà pour ce compte.
        ReputationAlreadyInitialized,
        /// Proposition de gouvernance introuvable.
        ProposalNotFound,
        /// Le compte a déjà voté sur cette proposition.
        AlreadyVoted,
        /// Seuil de votes insuffisant pour finaliser la proposition.
        ProposalThresholdNotMet,
        /// La proposition est déjà finalisée.
        ProposalAlreadyFinalized,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Hooks utilisés pour l'automatisation et le reporting.
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// Appel périodique pour ajuster automatiquement la réputation.
        fn on_finalize(_n: BlockNumberFor<T>) {
            let affected = Self::automated_reputation_adjustment();
            if affected > 0 {
                Self::deposit_event(Event::AutomatedReputationAdjustment(affected));
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Initialise la réputation du compte appelant.
        #[pallet::weight(10_000)]
        pub fn initialize_reputation(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(!Reputations::<T>::contains_key(&who), Error::<T>::ReputationAlreadyInitialized);
            let record = ReputationRecord {
                score: T::InitialReputation::get(),
                history: Vec::new(),
            };
            Reputations::<T>::insert(&who, record);
            Ok(())
        }

        /// Met à jour la réputation du compte appelant.
        /// Pour les ajustements négatifs, le delta est multiplié par le facteur de pénalité.
        #[pallet::weight(10_000)]
        pub fn update_reputation(origin: OriginFor<T>, delta: i32, reason: Vec<u8>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Reputations::<T>::try_mutate(&who, |maybe_record| -> DispatchResult {
                let record = maybe_record.as_mut().ok_or(Error::<T>::ReputationNotFound)?;
                let current = record.score as i32;
                let adjusted_delta = if delta < 0 {
                    delta.saturating_mul(PenaltyFactor::<T>::get() as i32)
                } else {
                    delta
                };
                let new_score = current.checked_add(adjusted_delta).ok_or(Error::<T>::ReputationUnderflow)?;
                ensure!(new_score >= 0, Error::<T>::ReputationUnderflow);
                record.score = new_score as u32;
                let now = <timestamp::Pallet<T>>::get();
                record.history.push(ReputationLog {
                    timestamp: now,
                    delta: adjusted_delta,
                    reason,
                });
                Self::deposit_event(Event::ReputationUpdated(who.clone(), adjusted_delta, record.score));
                Ok(())
            })
        }

        /// Permet à un utilisateur de proposer une mise à jour du facteur de pénalité.
        #[pallet::weight(10_000)]
        pub fn propose_parameter_update(origin: OriginFor<T>, new_value: u32, description: Vec<u8>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            // Extension potentielle : vérification d'identité via un module d'interopérabilité.
            let proposal_id = ProposalCount::<T>::get().checked_add(1).unwrap_or(1);
            let proposal = Proposal {
                id: proposal_id,
                proposer: who.clone(),
                proposal_type: ProposalType::UpdatePenaltyFactor,
                new_value,
                description,
                vote_count: 0,
                finalized: false,
            };
            Proposals::<T>::insert(proposal_id, proposal);
            ProposalCount::<T>::put(proposal_id);
            Self::deposit_event(Event::ProposalCreated(proposal_id, who));
            Ok(())
        }

        /// Permet à un utilisateur de voter pour une proposition.
        #[pallet::weight(10_000)]
        pub fn vote_on_proposal(origin: OriginFor<T>, proposal_id: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Proposals::<T>::try_mutate(proposal_id, |maybe_proposal| -> DispatchResult {
                let proposal = maybe_proposal.as_mut().ok_or(Error::<T>::ProposalNotFound)?;
                ensure!(!proposal.finalized, Error::<T>::ProposalAlreadyFinalized);
                ensure!(ProposalVotes::<T>::get(proposal_id, &who).is_none(), Error::<T>::AlreadyVoted);
                ProposalVotes::<T>::insert(proposal_id, &who, true);
                proposal.vote_count = proposal.vote_count.saturating_add(1);
                Self::deposit_event(Event::ProposalVoted(proposal_id, who, true));
                Ok(())
            })
        }

        /// Finalise une proposition si le seuil de votes est atteint.
        /// Cette extrinsèque est réservée à une origine de gouvernance.
        #[pallet::weight(10_000)]
        pub fn finalize_proposal(origin: OriginFor<T>, proposal_id: u32) -> DispatchResult {
            T::GovernanceOrigin::ensure_origin(origin)?;
            Proposals::<T>::try_mutate(proposal_id, |maybe_proposal| -> DispatchResult {
                let proposal = maybe_proposal.as_mut().ok_or(Error::<T>::ProposalNotFound)?;
                ensure!(!proposal.finalized, Error::<T>::ProposalAlreadyFinalized);
                ensure!(proposal.vote_count >= T::ProposalThreshold::get(), Error::<T>::ProposalThresholdNotMet);
                match proposal.proposal_type {
                    ProposalType::UpdatePenaltyFactor => {
                        PenaltyFactor::<T>::put(proposal.new_value);
                        Self::deposit_event(Event::GovernanceParameterUpdated(proposal.new_value));
                    }
                }
                proposal.finalized = true;
                Self::deposit_event(Event::ProposalFinalized(proposal_id, proposal.new_value));
                Ok(())
            })
        }
    }

    impl<T: Config> Pallet<T> {
        /// Ajuste automatiquement la réputation en fonction d'indicateurs d'activité (ici simulés).
        /// Retourne le nombre de comptes affectés.
        fn automated_reputation_adjustment() -> u32 {
            let mut affected = 0u32;
            for (account, mut record) in Reputations::<T>::iter() {
                if record.score < T::InitialReputation::get() {
                    record.score = record.score.saturating_add(1);
                    let now = <timestamp::Pallet<T>>::get();
                    record.history.push(ReputationLog {
                        timestamp: now,
                        delta: 1,
                        reason: b"Automated adjustment".to_vec(),
                    });
                    Reputations::<T>::insert(&account, record);
                    affected = affected.saturating_add(1);
                }
            }
            affected
        }
    }

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub initial_penalty_factor: u32,
        pub _marker: sp_std::marker::PhantomData<T>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                initial_penalty_factor: 1,
                _marker: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            PenaltyFactor::<T>::put(self.initial_penalty_factor);
        }
    }
}
