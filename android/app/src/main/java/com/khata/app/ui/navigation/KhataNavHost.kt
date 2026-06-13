package com.khata.app.ui.navigation

import android.net.Uri
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.navigation.NavDestination.Companion.hierarchy
import androidx.navigation.NavGraph.Companion.findStartDestination
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import com.khata.app.ui.accounts.AccountsScreen
import com.khata.app.ui.addtxn.AddTransactionScreen
import com.khata.app.ui.admin.AdminUsersScreen
import com.khata.app.ui.analytics.AnalyticsScreen
import com.khata.app.ui.auth.LoginScreen
import com.khata.app.ui.auth.ResetPasswordScreen
import com.khata.app.ui.auth.SetupScreen
import com.khata.app.ui.budgets.BudgetsScreen
import com.khata.app.ui.categories.CategoriesScreen
import com.khata.app.ui.chat.ChatScreen
import com.khata.app.ui.dashboard.DashboardScreen
import com.khata.app.ui.more.MoreItem
import com.khata.app.ui.more.MoreScreen
import com.khata.app.ui.portfolio.PortfolioScreen
import com.khata.app.ui.profile.ProfileScreen
import com.khata.app.ui.rules.RulesScreen
import com.khata.app.ui.theme.ThemeManager
import com.khata.app.ui.transactions.TransactionsScreen
import com.khata.app.ui.upload.CombinedUploadScreen
import com.khata.app.viewmodel.MainViewModel
import com.khata.app.viewmodel.TxnFilter
import kotlinx.coroutines.launch

sealed class Screen(val route: String, val label: String = "", val icon: @Composable () -> Unit = {}) {
    data object Setup : Screen("setup")
    data object Login : Screen("login")
    data object Dashboard : Screen("dashboard", "Dashboard", { Icon(Icons.Default.Home, contentDescription = null) })
    data object Transactions : Screen("transactions", "Txns", { Icon(Icons.Default.Receipt, contentDescription = null) })
    data object Chat : Screen("chat", "Claude", { Icon(Icons.Default.Chat, contentDescription = null) })
    data object Upload : Screen("upload", "Add", { Icon(Icons.Default.AddCircle, contentDescription = null) })
    data object Analytics : Screen("analytics", "Analytics", { Icon(Icons.Default.Analytics, contentDescription = null) })
    data object AdminUsers : Screen("admin_users")
    data object ResetPassword : Screen("reset_password")
    data object AddTransaction : Screen("add", "Add", { Icon(Icons.Default.AddCircle, contentDescription = null) })
    data object Accounts : Screen("accounts", "Accounts", { Icon(Icons.Default.AccountBalance, contentDescription = null) })
    data object Budgets : Screen("budgets", "Budgets", { Icon(Icons.Default.Savings, contentDescription = null) })
    data object Portfolio : Screen("portfolio", "Net Worth", { Icon(Icons.Default.MonetizationOn, contentDescription = null) })
    data object Rules : Screen("rules")
    data object Categories : Screen("categories", "Categories", { Icon(Icons.Default.Label, contentDescription = null) })
    data object Profile : Screen("profile", "Settings", { Icon(Icons.Default.Settings, contentDescription = null) })
    data object More : Screen("more", "More", { Icon(Icons.Default.MoreVert, contentDescription = null) })
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun KhataNavHost(themeManager: ThemeManager) {
    val navController = rememberNavController()
    val viewModel: MainViewModel = hiltViewModel()
    val authState by viewModel.authState.collectAsState()
    val dashboardState by viewModel.dashboardState.collectAsState()
    val txnState by viewModel.txnState.collectAsState()
    val chatState by viewModel.chatState.collectAsState()
    val usersState by viewModel.usersState.collectAsState()
    val accountsState by viewModel.accountsState.collectAsState()
    val rulesState by viewModel.rulesState.collectAsState()
    val budgetsState by viewModel.budgetsState.collectAsState()
    val portfolioState by viewModel.portfolioState.collectAsState()
    val categoriesState by viewModel.categoriesState.collectAsState()
    val cachedTxnsState by viewModel.cachedTxns.collectAsState()
    val context = LocalContext.current

    val isDark by themeManager.isDarkFlow.collectAsState(initial = false)
    val navBackStackEntry by navController.currentBackStackEntryAsState()
    val currentDestination = navBackStackEntry?.destination
    val scope = rememberCoroutineScope()

    val bottomNavItems = listOf(Screen.Dashboard, Screen.Transactions, Screen.Upload, Screen.More, Screen.Profile)
    val drawerItems = listOf(Screen.Chat, Screen.Accounts, Screen.Rules, Screen.Budgets, Screen.Portfolio, Screen.AdminUsers, Screen.Categories, Screen.ResetPassword)
    val showBottomBar = authState.isLoggedIn && currentDestination?.route != null && bottomNavItems.any { currentDestination?.route?.startsWith(it.route) == true }

    var uploadResult by remember { mutableStateOf<String?>(null) }

    val filePickerLauncher = rememberLauncherForActivityResult(
        contract = ActivityResultContracts.GetContent()
    ) { uri: Uri? -> uri?.let { viewModel.uploadStatement(context, it) { r -> uploadResult = r } } }

    LaunchedEffect(Unit) { viewModel.checkAuth() }

    if (authState.isChecking) { SplashScreen(); return }

    LaunchedEffect(authState.isLoggedIn, authState.setupRequired) {
        val dest = navController.currentDestination?.route
        when {
            authState.isLoggedIn && dest?.startsWith(Screen.Dashboard.route) != true -> navController.navigate(Screen.Dashboard.route) { popUpTo(0) { inclusive = true } }
            authState.setupRequired && dest != Screen.Setup.route -> navController.navigate(Screen.Setup.route) { popUpTo(0) { inclusive = true } }
            !authState.isLoggedIn && !authState.setupRequired && dest != Screen.Login.route -> navController.navigate(Screen.Login.route) { popUpTo(0) { inclusive = true } }
        }
    }

    Scaffold(
        bottomBar = {
            if (showBottomBar) {
                NavigationBar {
                    bottomNavItems.forEach { screen ->
                        NavigationBarItem(
                            icon = screen.icon, label = { Text(screen.label, fontSize = 10.sp) },
                            selected = currentDestination?.route?.startsWith(screen.route) == true,
                            onClick = { navController.navigate(screen.route) { popUpTo(navController.graph.findStartDestination().id) { saveState = true }; launchSingleTop = true; restoreState = true } }
                        )
                    }
                }
            }
        }
    ) { innerPadding ->
        NavHost(navController = navController, startDestination = Screen.Setup.route, modifier = Modifier.padding(innerPadding),
            enterTransition = { fadeIn(animationSpec = tween(150)) },
            exitTransition = { fadeOut(animationSpec = tween(150)) },
            popEnterTransition = { fadeIn(animationSpec = tween(150)) },
            popExitTransition = { fadeOut(animationSpec = tween(150)) }) {
            composable(Screen.Setup.route) { SetupScreen(isLoading = authState.isLoading, error = authState.error, onSetup = { e, p -> viewModel.setup(e, p) }) }
            composable(Screen.Login.route) { LoginScreen(isLoading = authState.isLoading, error = authState.error, onLogin = { e, p -> viewModel.login(e, p) }) }

            composable(Screen.Dashboard.route) { DashboardScreen(stats = dashboardState.stats, analysis = dashboardState.analysis, isLoading = dashboardState.isLoading, error = dashboardState.error, onRefresh = { viewModel.refreshDashboard() }, onNavigateToTransactions = { cat ->
                viewModel.updateTxnFilter(TxnFilter(category = cat))
                navController.navigate(Screen.Transactions.route)
            }) }

            composable(Screen.Upload.route) { CombinedUploadScreen(resultMessage = uploadResult, onPickFile = { filePickerLauncher.launch("*/*") }, onClearResult = { uploadResult = null }, onClearAllData = { viewModel.clearAllData { msg -> uploadResult = msg } }, onAddTxn = { viewModel.createTxn(it) }) }

            composable(Screen.Transactions.route) {
                val filterState by viewModel.txnFilterState.collectAsState()
                TransactionsScreen(
                    txnState = txnState.txns ?: cachedTxnsState,
                    categories = txnState.categories,
                    isLoading = txnState.isLoading,
                    error = txnState.error,
                    filter = filterState,
                    onLoad = { s, d, c, f, t, p ->
                        viewModel.loadTransactions(s, d, c, f, t, p)
                    },
                    onToggleTransfer = { id, v -> viewModel.toggleTransfer(id, v) },
                    onUpdateNotes = { id, n -> viewModel.updateNotes(id, n) },
                    onUpdateCategory = { id, cat -> viewModel.updateCategory(id, cat) }
                )
            }

            composable(Screen.Chat.route) { ChatScreen(messages = chatState.messages, isLoading = chatState.isLoading, error = chatState.error, onLoad = { viewModel.loadChatHistory() }, onSend = { q -> viewModel.sendChatMessage(q) }) }

            composable(Screen.Analytics.route) { AnalyticsScreen(stats = dashboardState.stats, analysis = dashboardState.analysis, isLoading = dashboardState.isLoading, onRefresh = { viewModel.refreshDashboard() }) }

            composable(Screen.AddTransaction.route) { AddTransactionScreen(isLoading = authState.isLoading, error = authState.error, onAdd = { viewModel.createTxn(it) }, onBack = { navController.popBackStack() }) }

            composable(Screen.Accounts.route) { AccountsScreen(accounts = accountsState.accounts, isLoading = accountsState.isLoading, error = accountsState.error, onLoad = { viewModel.loadAccounts() }, onCreate = { l, i -> viewModel.createAccount(l, i) }, onDelete = { id -> viewModel.deleteAccount(id) }) }

            composable(Screen.Rules.route) { RulesScreen(rules = rulesState.rules, isLoading = rulesState.isLoading, error = rulesState.error, onLoad = { viewModel.loadRules() }, onCreate = { p, c -> viewModel.createRule(p, c) }, onDelete = { id -> viewModel.deleteRule(id) }, onApply = { viewModel.applyRules() }) }

            composable(Screen.Budgets.route) { BudgetsScreen(budgets = budgetsState.budgets, status = budgetsState.status, isLoading = budgetsState.isLoading, error = budgetsState.error, onLoad = { viewModel.loadBudgets() }, onCreate = { c, l -> viewModel.createBudget(c, l) }, onDelete = { id -> viewModel.deleteBudget(id) }) }

            composable(Screen.Portfolio.route) { PortfolioScreen(snapshot = portfolioState.snapshot, isLoading = portfolioState.isLoading, error = portfolioState.error, onLoad = { viewModel.loadPortfolio() }, onCreateAsset = { n, t, v -> viewModel.createAsset(n, t, v) }, onDeleteAsset = { id -> viewModel.deleteAsset(id) }, onCreateLiability = { n, t, v -> viewModel.createLiability(n, t, v) }, onDeleteLiability = { id -> viewModel.deleteLiability(id) }) }

            composable(Screen.Categories.route) { CategoriesScreen(categories = categoriesState.list, isLoading = categoriesState.isLoading, error = categoriesState.error, onLoad = { viewModel.loadCategories() }, onCreate = { n, t, c, d -> viewModel.createCategory(n, t, c, d) }, onDelete = { id -> viewModel.deleteCategory(id) }) }

            composable(Screen.Profile.route) { ProfileScreen(user = authState.user, isDark = isDark, onToggleDark = { scope.launch { themeManager.setDark(!isDark) } }, onResetPassword = { navController.navigate(Screen.ResetPassword.route) }, onClearAllData = { viewModel.clearAllData { msg -> } }, onUpdateEmail = { email -> viewModel.updateEmail(email) }, onLogout = { viewModel.logout() }) }

            composable(Screen.More.route) { MoreScreen(items = listOf(
                MoreItem("Analytics", Screen.Analytics.route) { Icon(Icons.Default.Analytics, contentDescription = null, modifier = Modifier.size(24.dp), tint = MaterialTheme.colorScheme.primary) },
                MoreItem("Chat", Screen.Chat.route) { Icon(Icons.Default.Chat, contentDescription = null, modifier = Modifier.size(24.dp), tint = MaterialTheme.colorScheme.primary) },
                MoreItem("Accounts", Screen.Accounts.route) { Icon(Icons.Default.AccountBalance, contentDescription = null, modifier = Modifier.size(24.dp), tint = MaterialTheme.colorScheme.primary) },
                MoreItem("Rules", Screen.Rules.route) { Icon(Icons.Default.Tag, contentDescription = null, modifier = Modifier.size(24.dp), tint = MaterialTheme.colorScheme.primary) },
                MoreItem("Budgets", Screen.Budgets.route) { Icon(Icons.Default.Savings, contentDescription = null, modifier = Modifier.size(24.dp), tint = MaterialTheme.colorScheme.primary) },
                MoreItem("Portfolio", Screen.Portfolio.route) { Icon(Icons.Default.MonetizationOn, contentDescription = null, modifier = Modifier.size(24.dp), tint = MaterialTheme.colorScheme.primary) },
                MoreItem("Categories", Screen.Categories.route) { Icon(Icons.Default.Label, contentDescription = null, modifier = Modifier.size(24.dp), tint = MaterialTheme.colorScheme.primary) },
                MoreItem("Users", Screen.AdminUsers.route) { Icon(Icons.Default.People, contentDescription = null, modifier = Modifier.size(24.dp), tint = MaterialTheme.colorScheme.primary) },
            ), onNavigate = { route -> navController.navigate(route) }) }

            composable(Screen.AdminUsers.route) { AdminUsersScreen(users = usersState.users, isLoading = usersState.isLoading, error = usersState.error, success = usersState.success, onLoad = { viewModel.loadUsers() }, onCreateUser = { e, p -> viewModel.createUser(e, p) }, onDeleteUser = { id -> viewModel.deleteUser(id) }) }

            composable(Screen.ResetPassword.route) { ResetPasswordScreen(isLoading = authState.isLoading, error = authState.error, onReset = { c, n -> viewModel.resetPassword(c, n) { navController.popBackStack() } }, onBack = { navController.popBackStack() }) }
        }
    }
}

@Composable
private fun SplashScreen() {
    Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
        Column(horizontalAlignment = Alignment.CenterHorizontally) {
            Surface(Modifier.size(72.dp), shape = RoundedCornerShape(20.dp), color = MaterialTheme.colorScheme.primary, shadowElevation = 8.dp) { Box(contentAlignment = Alignment.Center) { Text("₹", fontSize = 36.sp, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.onPrimary) } }
            Spacer(Modifier.height(16.dp)); Text("Khata", fontSize = 28.sp, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.onBackground)
            Spacer(Modifier.height(24.dp)); CircularProgressIndicator(Modifier.size(28.dp), strokeWidth = 3.dp, color = MaterialTheme.colorScheme.primary)
        }
    }
}

@Composable
private fun UploadScreen(isDark: Boolean, onToggleDark: () -> Unit, resultMessage: String?, onPickFile: () -> Unit, onClearResult: () -> Unit, onClearAllData: () -> Unit) {
    var showClearDialog by remember { mutableStateOf(false) }
    if (showClearDialog) { AlertDialog(onDismissRequest = { showClearDialog = false }, title = { Text("Clear All Data?") }, text = { Text("This action cannot be undone.") }, confirmButton = { TextButton(onClick = { showClearDialog = false; onClearAllData() }) { Text("Clear", color = MaterialTheme.colorScheme.error) } }, dismissButton = { TextButton(onClick = { showClearDialog = false }) { Text("Cancel") } }) }
    Column(Modifier.fillMaxSize().padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
            Column { Text("Upload Statement", style = MaterialTheme.typography.headlineSmall); Text("CSV or Excel", color = MaterialTheme.colorScheme.onSurfaceVariant) }
            IconButton(onClick = onToggleDark) { Icon(if (isDark) Icons.Default.LightMode else Icons.Default.DarkMode, contentDescription = "Theme") }
        }
        Surface(onClick = onPickFile, modifier = Modifier.fillMaxWidth().height(180.dp), shape = MaterialTheme.shapes.medium, color = MaterialTheme.colorScheme.surface, tonalElevation = 2.dp) { Column(Modifier.fillMaxSize().padding(24.dp), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.Center) { Icon(Icons.Default.UploadFile, contentDescription = null, modifier = Modifier.size(48.dp), tint = MaterialTheme.colorScheme.primary); Spacer(Modifier.height(12.dp)); Text("Tap to select"); Text("CSV, XLS, XLSX", color = MaterialTheme.colorScheme.onSurfaceVariant) } }
        OutlinedButton(onClick = { showClearDialog = true }, modifier = Modifier.fillMaxWidth(), colors = ButtonDefaults.outlinedButtonColors(contentColor = MaterialTheme.colorScheme.error)) { Icon(Icons.Default.DeleteForever, contentDescription = null, modifier = Modifier.size(18.dp)); Spacer(Modifier.width(6.dp)); Text("Clear All Data") }
        Spacer(Modifier.weight(1f))
        resultMessage?.let { msg -> Surface(shape = MaterialTheme.shapes.medium, color = if (msg.startsWith("Error")) MaterialTheme.colorScheme.errorContainer else MaterialTheme.colorScheme.secondaryContainer) { Text(msg, modifier = Modifier.padding(12.dp), fontSize = 13.sp) }; if (!msg.startsWith("Error")) { Spacer(Modifier.height(4.dp)); TextButton(onClick = onClearResult) { Text("Dismiss") } } }
    }
}
