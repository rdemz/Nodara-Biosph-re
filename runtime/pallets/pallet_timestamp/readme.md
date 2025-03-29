# Nodara BIOSPHÈRE QUANTIC Runtime

Ce repository contient le runtime complet du réseau **Nodara BIOSPHÈRE QUANTIC**. Le runtime intègre l'ensemble des modules (pallets) personnalisés et standard, et expose une API runtime complète pour interagir avec la blockchain.

## Modules Intégrés

- **System** : Gestion de base du runtime Substrate.
- **Timestamp** : Fourniture d'horodatages fiables (via `pallet_timestamp`).
- **Aura** : Mécanisme de production de blocs.
- **Grandpa** : Finalisation des blocs.
- **Session** : Gestion des sessions de validation.
- **Bridge** : Module de bridge inter‑chaînes.
- **Biosphere** : Gestion de l'état adaptatif du réseau.
- **Growth** : Gestion des incentives de croissance.
- **Identity** : Gestion des identités décentralisées.
- **Interop** : Interopérabilité avec des chaînes externes.
- **IoTBridge** : Pont IoT pour la collecte de données.
- **LiquidityFlow** : Gestion des flux de liquidité.
- **RewardEngine** : Distribution dynamique des récompenses.
- **StabilityGuard** : Gestion dynamique de la stabilité du réseau.
- **Standards** : Application des standards techniques et réglementaires.
- **Pow** : Mécanisme Proof-of-Work biomimétique.
- **PredictiveGuard** : Ajustements prédictifs basés sur des signaux économiques.
- **Reputation** : Système de réputation pour les comptes.
- **ReserveFund** : Gestion du fonds de réserve.
- **Marketplace** : Marketplace décentralisée.

## Runtime API

Le runtime expose une API complète via le trait `NodeRuntimeApi` permettant d'interroger l'état de chacun des modules :
- `marketplace_get_asset(asset_id: u64)`
- `biosphere_get_state()`
- `growth_get_state()`
- `identity_get(account: u64)`
- `interop_get_history()`
- `iot_get_record(message_id: u64)`
- `liquidity_get_state()`
- `reward_get_state()`
- `stability_get_state()`
- `standards_get_standard(standard_id: Vec<u8>)`
- `pow_get_state()`
- `predictive_get_value()`
- `reputation_get(account: u64)`
- `reserve_get_state()`
- `dummy()`

## Installation et Compilation

### Prérequis

- Rust (stable)
- Cargo
- Les outils Substrate (par exemple, `substrate` CLI et `cargo-contract` si nécessaire)

### Compilation

Pour compiler le runtime, exécutez la commande :

```bash
cargo build --release --workspace
