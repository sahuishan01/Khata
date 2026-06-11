package com.khata.app.viewmodel

import android.content.Context
import android.net.Uri
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.khata.app.api.*
import com.khata.app.data.KhataRepository
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch
import okhttp3.MediaType.Companion.toMediaTypeOrNull
import okhttp3.MultipartBody
import okhttp3.RequestBody.Companion.toRequestBody
import javax.inject.Inject

data class AuthUiState(
    val isLoading: Boolean = false,
    val isLoggedIn: Boolean = false,
    val setupRequired: Boolean = false,
    val user: MeResponse? = null,
    val error: String? = null
)

data class DashboardUiState(
    val stats: DashboardStats? = null,
    val analysis: AnalysisStats? = null,
    val isLoading: Boolean = false,
    val error: String? = null
)

data class TxnUiState(
    val txns: TxnListResponse? = null,
    val categories: List<String> = emptyList(),
    val isLoading: Boolean = false,
    val error: String? = null
)

data class ChatUiState(
    val messages: List<ChatHistoryResponse> = emptyList(),
    val isLoading: Boolean = false,
    val error: String? = null
)

data class UsersUiState(
    val users: List<UserResponse> = emptyList(),
    val isLoading: Boolean = false,
    val error: String? = null,
    val success: String? = null
)

@HiltViewModel
class MainViewModel @Inject constructor(
    private val repository: KhataRepository
) : ViewModel() {

    private val _authState = MutableStateFlow(AuthUiState())
    val authState: StateFlow<AuthUiState> = _authState.asStateFlow()

    private val _dashboardState = MutableStateFlow(DashboardUiState())
    val dashboardState: StateFlow<DashboardUiState> = _dashboardState.asStateFlow()

    private val _txnState = MutableStateFlow(TxnUiState())
    val txnState: StateFlow<TxnUiState> = _txnState.asStateFlow()

    private val _chatState = MutableStateFlow(ChatUiState())
    val chatState: StateFlow<ChatUiState> = _chatState.asStateFlow()

    private val _usersState = MutableStateFlow(UsersUiState())
    val usersState: StateFlow<UsersUiState> = _usersState.asStateFlow()

    fun checkAuth() {
        viewModelScope.launch {
            try {
                val setupRequired = repository.checkSetupStatus()
                if (!setupRequired) {
                    try {
                        val user = repository.getMe()
                        _authState.value = AuthUiState(isLoggedIn = true, user = user)
                    } catch (_: Exception) {
                        _authState.value = AuthUiState(setupRequired = false)
                    }
                } else {
                    _authState.value = AuthUiState(setupRequired = true)
                }
            } catch (_: Exception) {
                _authState.value = AuthUiState(setupRequired = true)
            }
        }
    }

    fun login(email: String, password: String) {
        viewModelScope.launch {
            _authState.value = _authState.value.copy(isLoading = true, error = null)
            try {
                repository.login(email, password)
                val user = repository.getMe()
                _authState.value = AuthUiState(isLoggedIn = true, user = user)
            } catch (e: Exception) {
                _authState.value = _authState.value.copy(
                    isLoading = false,
                    error = e.message ?: "Login failed"
                )
            }
        }
    }

    fun setup(email: String, password: String) {
        viewModelScope.launch {
            _authState.value = _authState.value.copy(isLoading = true, error = null)
            try {
                repository.setup(email, password)
                val user = repository.getMe()
                _authState.value = AuthUiState(isLoggedIn = true, user = user)
            } catch (e: Exception) {
                _authState.value = _authState.value.copy(
                    isLoading = false,
                    error = e.message ?: "Setup failed"
                )
            }
        }
    }

    fun logout() {
        viewModelScope.launch {
            repository.logout()
            _authState.value = AuthUiState()
        }
    }

    fun refreshDashboard() {
        viewModelScope.launch {
            _dashboardState.value = _dashboardState.value.copy(isLoading = true, error = null)
            try {
                val stats = repository.getDashboard()
                val analysis = repository.getAnalysis()
                _dashboardState.value = DashboardUiState(stats = stats, analysis = analysis)
            } catch (e: Exception) {
                _dashboardState.value = _dashboardState.value.copy(
                    isLoading = false,
                    error = e.message
                )
            }
        }
    }

    fun loadTransactions(
        sortBy: String = "date",
        sortDir: String = "desc",
        category: String? = null,
        from: String? = null,
        to: String? = null
    ) {
        viewModelScope.launch {
            _txnState.value = _txnState.value.copy(isLoading = true, error = null)
            try {
                val txns = repository.listTxns(sortBy = sortBy, sortDir = sortDir, category = category, from = from, to = to)
                val cats = repository.listCategories()
                _txnState.value = TxnUiState(txns = txns, categories = cats)
            } catch (e: Exception) {
                _txnState.value = _txnState.value.copy(
                    isLoading = false,
                    error = e.message ?: "Failed to load transactions"
                )
            }
        }
    }

    fun loadChatHistory() {
        viewModelScope.launch {
            _chatState.value = _chatState.value.copy(isLoading = true, error = null)
            try {
                val messages = repository.getChatHistory()
                _chatState.value = ChatUiState(messages = messages)
            } catch (e: Exception) {
                _chatState.value = _chatState.value.copy(
                    isLoading = false,
                    error = e.message ?: "Failed to load chat"
                )
            }
        }
    }

    fun sendChatMessage(question: String) {
        viewModelScope.launch {
            val tempMsg = ChatHistoryResponse(
                id = System.currentTimeMillis().toString(),
                role = "user", content = question, sqlUsed = null
            )
            _chatState.value = _chatState.value.copy(
                messages = _chatState.value.messages + tempMsg,
                isLoading = true,
                error = null
            )
            try {
                val response = repository.askChat(question)
                val reply = ChatHistoryResponse(
                    id = (System.currentTimeMillis() + 1).toString(),
                    role = "assistant", content = response.answer, sqlUsed = response.sqlUsed
                )
                _chatState.value = _chatState.value.copy(
                    messages = _chatState.value.messages + reply,
                    isLoading = false
                )
            } catch (e: Exception) {
                val errMsg = ChatHistoryResponse(
                    id = (System.currentTimeMillis() + 1).toString(),
                    role = "assistant",
                    content = "Error: ${e.message ?: "Failed to get response"}",
                    sqlUsed = null
                )
                _chatState.value = _chatState.value.copy(
                    messages = _chatState.value.messages + errMsg,
                    isLoading = false,
                    error = e.message
                )
            }
        }
    }

    fun loadUsers() {
        viewModelScope.launch {
            _usersState.value = _usersState.value.copy(isLoading = true, error = null, success = null)
            try {
                val users = repository.listUsers()
                _usersState.value = UsersUiState(users = users)
            } catch (e: Exception) {
                _usersState.value = _usersState.value.copy(
                    isLoading = false,
                    error = e.message ?: "Failed to load users"
                )
            }
        }
    }

    fun createUser(email: String, password: String) {
        viewModelScope.launch {
            _usersState.value = _usersState.value.copy(isLoading = true, error = null, success = null)
            try {
                repository.createUser(email, password)
                _usersState.value = _usersState.value.copy(
                    isLoading = false,
                    success = "User $email created"
                )
                loadUsers()
            } catch (e: Exception) {
                _usersState.value = _usersState.value.copy(
                    isLoading = false,
                    error = e.message ?: "Failed to create user"
                )
            }
        }
    }

    fun deleteUser(id: String) {
        viewModelScope.launch {
            try {
                repository.deleteUser(id)
                loadUsers()
            } catch (e: Exception) {
                _usersState.value = _usersState.value.copy(
                    error = e.message ?: "Failed to delete user"
                )
            }
        }
    }

    fun resetPassword(current: String, newPassword: String, onSuccess: () -> Unit) {
        viewModelScope.launch {
            _authState.value = _authState.value.copy(isLoading = true, error = null)
            try {
                repository.resetPassword(current, newPassword)
                _authState.value = _authState.value.copy(isLoading = false)
                onSuccess()
            } catch (e: Exception) {
                _authState.value = _authState.value.copy(
                    isLoading = false,
                    error = e.message ?: "Failed to reset password"
                )
            }
        }
    }

    fun uploadStatement(context: Context, uri: Uri, onResult: (String) -> Unit) {
        viewModelScope.launch {
            try {
                val inputStream = context.contentResolver.openInputStream(uri)
                    ?: return@launch onResult("Error: Cannot read file")
                val bytes = inputStream.readBytes()
                inputStream.close()

                val fileName = getFileName(context, uri) ?: "upload_${System.currentTimeMillis()}"
                val requestBody = bytes.toRequestBody("application/octet-stream".toMediaTypeOrNull())
                val part = MultipartBody.Part.createFormData("file", fileName, requestBody)

                repository.uploadStatement(part)
                onResult("Statement uploaded successfully!")
            } catch (e: Exception) {
                onResult("Error: ${e.message ?: "Upload failed"}")
            }
        }
    }

    private fun getFileName(context: Context, uri: Uri): String? {
        var name: String? = null
        val cursor = context.contentResolver.query(uri, null, null, null, null)
        cursor?.use {
            if (it.moveToFirst()) {
                val idx = it.getColumnIndex(android.provider.OpenableColumns.DISPLAY_NAME)
                if (idx >= 0) name = it.getString(idx)
            }
        }
        return name
    }

    fun clearAllData(onResult: (String) -> Unit) {
        viewModelScope.launch {
            try {
                repository.clearAllData()
                onResult("All data cleared successfully!")
            } catch (e: Exception) {
                onResult("Error: ${e.message ?: "Failed to clear data"}")
            }
        }
    }
}
