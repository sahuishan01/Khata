package com.khata.app.ui.admin

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Visibility
import androidx.compose.material.icons.filled.VisibilityOff
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.khata.app.ui.components.shared.ButtonVariant
import com.khata.app.ui.components.shared.KhataButton
import com.khata.app.ui.components.shared.KhataCard
import com.khata.app.ui.components.shared.KhataCardBody
import com.khata.app.ui.components.shared.KhataCardHeader
import com.khata.app.ui.components.shared.KhataEmptyState
import com.khata.app.ui.components.shared.KhataField
import com.khata.app.ui.components.shared.KhataListRow
import com.khata.app.ui.components.shared.KhataListRowText

@Composable
fun AdminUsersScreen(
    users: List<com.khata.app.api.UserResponse>,
    isLoading: Boolean,
    error: String?,
    success: String?,
    onLoad: () -> Unit,
    onCreateUser: (String, String) -> Unit,
    onDeleteUser: (String) -> Unit
) {
    LaunchedEffect(Unit) { onLoad() }

    var email by remember { mutableStateOf("") }
    var password by remember { mutableStateOf("") }
    var showPwd by remember { mutableStateOf(false) }

    LazyColumn(
        modifier = Modifier.fillMaxSize().padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        item {
            Text("Manage Users", fontSize = 22.sp, fontWeight = FontWeight.Bold)
            Text("Add and manage user accounts", fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }

        item {
            KhataCard(Modifier.fillMaxWidth()) {
                KhataCardHeader("Add User")
                KhataCardBody {
                    KhataField(
                        value = email,
                        onValueChange = { email = it },
                        placeholder = "Email"
                    )
                    Spacer(Modifier.height(8.dp))

                    OutlinedTextField(
                        value = password,
                        onValueChange = { password = it },
                        placeholder = { Text("Password", fontSize = 11.sp) },
                        singleLine = true,
                        visualTransformation = if (showPwd) VisualTransformation.None else PasswordVisualTransformation(),
                        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password),
                        modifier = Modifier.fillMaxWidth(),
                        shape = RoundedCornerShape(9.dp),
                        textStyle = LocalTextStyle.current.copy(fontSize = 11.sp),
                        trailingIcon = {
                            IconButton(onClick = { showPwd = !showPwd }) {
                                Icon(if (showPwd) Icons.Default.VisibilityOff else Icons.Default.Visibility, contentDescription = null)
                            }
                        }
                    )

                    if (error != null) {
                        Spacer(Modifier.height(8.dp))
                        Text(error, fontSize = 13.sp, color = MaterialTheme.colorScheme.error)
                    }
                    if (success != null) {
                        Spacer(Modifier.height(8.dp))
                        Text(success, fontSize = 13.sp, color = MaterialTheme.colorScheme.secondary)
                    }

                    Spacer(Modifier.height(12.dp))
                    KhataButton(
                        onClick = { onCreateUser(email, password); email = ""; password = "" },
                        enabled = !isLoading && email.isNotBlank() && password.length >= 8,
                        modifier = Modifier.fillMaxWidth()
                    ) {
                        if (isLoading) CircularProgressIndicator(Modifier.size(18.dp), strokeWidth = 2.dp)
                        else Text("Add User")
                    }
                }
            }
        }

        item {
            KhataCard(Modifier.fillMaxWidth()) {
                KhataCardHeader("Users")
                if (users.isEmpty()) {
                    KhataEmptyState(
                        emoji = "\uD83D\uDC65",
                        title = "No users yet",
                        description = "Add users using the form above to grant them access."
                    )
                } else {
                    users.forEach { user ->
                        KhataListRow(
                            content = { KhataListRowText(primary = user.email, secondary = user.role) },
                            trailing = {
                                KhataButton(onClick = { onDeleteUser(user.id) }, variant = ButtonVariant.Ghost) {
                                    Icon(Icons.Default.Delete, "Delete", tint = MaterialTheme.colorScheme.error)
                                }
                            }
                        )
                    }
                }
            }
        }
    }
}
