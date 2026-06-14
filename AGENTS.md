# Khata Development Guidelines

## Feature Parity

Changes made to the web frontend should also be applied to the Android app, and vice versa. Before considering a feature complete, verify it works on both platforms:
- `/profile` (web) ↔ `ProfileScreen.kt` (Android)
- `/upload` (web) ↔ `CombinedUploadScreen.kt` (Android)  
- `/transactions` (web) ↔ `TransactionsScreen.kt` (Android)
- `/accounts` (web) ↔ `AccountsScreen.kt` (Android)
- `/rules` (web) ↔ `RulesScreen.kt` (Android)
- `/budgets` (web) ↔ `BudgetsScreen.kt` (Android)
- `/portfolio` (web) ↔ `PortfolioScreen.kt` (Android)
- `/chat` (web) ↔ `ChatScreen.kt` (Android)
- `/admin/users` (web) ↔ `AdminUsersScreen.kt` (Android)
- `/categories` (web) ↔ `CategoriesScreen.kt` (Android)
- `/more` (web) ↔ `MoreScreen.kt` (Android)

Keep the app version below 1.0 (pre-release, e.g. 0.x.y) until the user explicitly says it's production-ready.

Always ask if the previous build was successful before changing the version number. The last successful build tag is the reference point.

After the user confirms a build was successful, auto-update the version number stored here and create the next release tag. The current version is defined by the latest git tag matching `v*`.


## Design
- Read DESIGN.md for design related instructions

## Security
- Read SECURITY.md for security related instructions
