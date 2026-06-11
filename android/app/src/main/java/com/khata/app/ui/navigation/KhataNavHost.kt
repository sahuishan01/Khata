package com.khata.app.ui.navigation

import androidx.compose.foundation.layout.*
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.navigation.NavDestination.Companion.hierarchy
import androidx.navigation.NavGraph.Companion.findStartDestination
import androidx.navigation.NavType
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import androidx.navigation.navArgument
import com.khata.app.viewmodel.MainViewModel
import com.khata.app.ui.admin.AdminUsersScreen
import com.khata.app.ui.auth.LoginScreen
import com.khata.app.ui.auth.ResetPasswordScreen
import com.khata.app.ui.auth.SetupScreen
import com.khata.app.ui.dashboard.DashboardScreen

sealed class Screen(val route: String, val label: String = "", val icon: @Composable () -> Unit = {}) {
    data object Setup : Screen("setup")
    data object Login : Screen("login")
    data object Dashboard : Screen("dashboard", "Dashboard", { Icon(Icons.Default.Home, contentDescription = null) })
    data object Transactions : Screen("transactions", "Transactions", { Icon(Icons.Default.Receipt, contentDescription = null) })
    data object Chat : Screen("chat", "Ask Claude", { Icon(Icons.Default.Chat, contentDescription = null) })
    data object AdminUsers : Screen("admin_users")
    data object ResetPassword : Screen("reset_password")
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun KhataNavHost() {
    val navController = rememberNavController()
    val viewModel: MainViewModel = hiltViewModel()
    val authState by viewModel.authState.collectAsState()
    val dashboardState by viewModel.dashboardState.collectAsState()
    val usersState by viewModel.usersState.collectAsState()

    val navBackStackEntry by navController.currentBackStackEntryAsState()
    val currentDestination = navBackStackEntry?.destination

    val bottomNavItems = listOf(Screen.Dashboard, Screen.Transactions, Screen.Chat)
    val showBottomBar = authState.isLoggedIn && currentDestination?.route in bottomNavItems.map { it.route }

    LaunchedEffect(authState.isLoggedIn, authState.setupRequired) {
        when {
            authState.setupRequired -> navController.navigate(Screen.Setup.route) { popUpTo(0) { inclusive = true } }
            !authState.isLoggedIn -> navController.navigate(Screen.Login.route) { popUpTo(0) { inclusive = true } }
            else -> navController.navigate(Screen.Dashboard.route) { popUpTo(0) { inclusive = true } }
        }
    }

    Scaffold(
        bottomBar = {
            if (showBottomBar) {
                NavigationBar {
                    bottomNavItems.forEach { screen ->
                        NavigationBarItem(
                            icon = screen.icon,
                            label = { Text(screen.label, fontSize = MaterialTheme.typography.labelSmall.fontSize) },
                            selected = currentDestination?.hierarchy?.any { it.route == screen.route } == true,
                            onClick = {
                                navController.navigate(screen.route) {
                                    popUpTo(navController.graph.findStartDestination().id) { saveState = true }
                                    launchSingleTop = true
                                    restoreState = true
                                }
                            }
                        )
                    }
                }
            }
        }
    ) { innerPadding ->
        NavHost(
            navController = navController,
            startDestination = Screen.Setup.route,
            modifier = Modifier.padding(innerPadding)
        ) {
            composable(Screen.Setup.route) {
                SetupScreen(
                    isLoading = authState.isLoading,
                    error = authState.error,
                    onSetup = { email, password -> viewModel.setup(email, password) }
                )
            }

            composable(Screen.Login.route) {
                LoginScreen(
                    isLoading = authState.isLoading,
                    error = authState.error,
                    onLogin = { email, password -> viewModel.login(email, password) }
                )
            }

            composable(Screen.Dashboard.route) {
                DashboardScreen(
                    stats = dashboardState.stats,
                    analysis = dashboardState.analysis,
                    isLoading = dashboardState.isLoading,
                    error = dashboardState.error,
                    onRefresh = { viewModel.refreshDashboard() },
                    onNavigateToTransactions = {}
                )
            }

            composable(Screen.Transactions.route) {
                Box(modifier = Modifier.fillMaxSize().padding(16.dp)) {
                    Text("Transactions", style = MaterialTheme.typography.headlineSmall)
                }
            }

            composable(Screen.Chat.route) {
                Box(modifier = Modifier.fillMaxSize().padding(16.dp)) {
                    Text("Ask Claude", style = MaterialTheme.typography.headlineSmall)
                }
            }

            composable(Screen.AdminUsers.route) {
                AdminUsersScreen(
                    users = usersState.users,
                    isLoading = usersState.isLoading,
                    error = usersState.error,
                    success = usersState.success,
                    onLoad = { viewModel.loadUsers() },
                    onCreateUser = { email, password -> viewModel.createUser(email, password) },
                    onDeleteUser = { id -> viewModel.deleteUser(id) }
                )
            }

            composable(Screen.ResetPassword.route) {
                ResetPasswordScreen(
                    isLoading = authState.isLoading,
                    error = authState.error,
                    onReset = { current, newPassword ->
                        viewModel.resetPassword(current, newPassword) {
                            navController.popBackStack()
                        }
                    },
                    onBack = { navController.popBackStack() }
                )
            }
        }
    }
}
