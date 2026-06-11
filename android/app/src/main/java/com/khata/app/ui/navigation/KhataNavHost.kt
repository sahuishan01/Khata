package com.khata.app.ui.navigation

import android.net.Uri
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.*
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.navigation.NavDestination.Companion.hierarchy
import androidx.navigation.NavGraph.Companion.findStartDestination
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import com.khata.app.ui.admin.AdminUsersScreen
import com.khata.app.ui.auth.LoginScreen
import com.khata.app.ui.auth.ResetPasswordScreen
import com.khata.app.ui.auth.SetupScreen
import com.khata.app.ui.dashboard.DashboardScreen
import com.khata.app.ui.theme.ThemeManager
import com.khata.app.viewmodel.MainViewModel

sealed class Screen(val route: String, val label: String = "", val icon: @Composable () -> Unit = {}) {
    data object Setup : Screen("setup")
    data object Login : Screen("login")
    data object Dashboard : Screen("dashboard", "Dashboard", { Icon(Icons.Default.Home, contentDescription = null) })
    data object Transactions : Screen("transactions", "Transactions", { Icon(Icons.Default.Receipt, contentDescription = null) })
    data object Chat : Screen("chat", "Ask Claude", { Icon(Icons.Default.Chat, contentDescription = null) })
    data object Upload : Screen("upload", "Upload", { Icon(Icons.Default.UploadFile, contentDescription = null) })
    data object AdminUsers : Screen("admin_users")
    data object ResetPassword : Screen("reset_password")
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun KhataNavHost(themeManager: ThemeManager) {
    val navController = rememberNavController()
    val viewModel: MainViewModel = hiltViewModel()
    val authState by viewModel.authState.collectAsState()
    val dashboardState by viewModel.dashboardState.collectAsState()
    val usersState by viewModel.usersState.collectAsState()
    val context = LocalContext.current

    val isDark by themeManager.isDarkFlow.collectAsState(initial = false)

    val navBackStackEntry by navController.currentBackStackEntryAsState()
    val currentDestination = navBackStackEntry?.destination

    val bottomNavItems = listOf(Screen.Dashboard, Screen.Upload, Screen.Transactions, Screen.Chat)
    val showBottomBar = authState.isLoggedIn && currentDestination?.route in bottomNavItems.map { it.route }

    var uploadResult by remember { mutableStateOf<String?>(null) }

    val filePickerLauncher = rememberLauncherForActivityResult(
        contract = ActivityResultContracts.GetContent()
    ) { uri: Uri? ->
        uri?.let {
            viewModel.uploadStatement(context, it) { result ->
                uploadResult = result
            }
        }
    }

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

            composable(Screen.Upload.route) {
                UploadScreen(
                    isDark = isDark,
                    onToggleDark = { themeManager.setDark(!isDark) },
                    resultMessage = uploadResult,
                    onPickFile = { filePickerLauncher.launch("*/*") },
                    onClearResult = { uploadResult = null }
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

@Composable
private fun UploadScreen(
    isDark: Boolean,
    onToggleDark: () -> Unit,
    resultMessage: String?,
    onPickFile: () -> Unit,
    onClearResult: () -> Unit
) {
    Column(
        modifier = Modifier.fillMaxSize().padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween
        ) {
            Column {
                Text("Upload Statement", style = MaterialTheme.typography.headlineSmall)
                Text("CSV or Excel files", color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
            IconButton(onClick = onToggleDark) {
                Icon(
                    if (isDark) Icons.Default.LightMode else Icons.Default.DarkMode,
                    contentDescription = "Toggle theme"
                )
            }
        }

        Card(
            onClick = onPickFile,
            modifier = Modifier.fillMaxWidth().height(200.dp)
        ) {
            Column(
                modifier = Modifier.fillMaxSize().padding(24.dp),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.Center
            ) {
                Icon(
                    Icons.Default.UploadFile,
                    contentDescription = null,
                    modifier = Modifier.size(48.dp),
                    tint = MaterialTheme.colorScheme.primary
                )
                Spacer(Modifier.height(12.dp))
                Text("Tap to select a file", style = MaterialTheme.typography.titleMedium)
                Text("CSV, XLS, XLSX", color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        }

        resultMessage?.let { msg ->
            Card(
                colors = CardDefaults.cardColors(
                    containerColor = if (msg.startsWith("Error"))
                        MaterialTheme.colorScheme.errorContainer
                    else
                        MaterialTheme.colorScheme.secondaryContainer
                )
            ) {
                Text(msg, modifier = Modifier.padding(12.dp), fontSize = 13.sp)
            }
            if (!msg.startsWith("Error")) {
                Spacer(Modifier.height(4.dp))
                TextButton(onClick = onClearResult) {
                    Text("Dismiss")
                }
            }
        }
    }
}
