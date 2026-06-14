package com.khata.app.ui.analytics

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.khata.app.api.AnalysisStats
import com.khata.app.api.DashboardStats
import com.khata.app.ui.charts.MonthlyBarChart
import com.khata.app.ui.components.shared.KhataAmount
import com.khata.app.ui.components.shared.KhataCard
import com.khata.app.ui.components.shared.KhataCardBody
import com.khata.app.ui.components.shared.KhataCardHeader
import com.khata.app.ui.theme.KhataColors
import com.khata.app.util.formatINR

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AnalyticsDetailScreen(
    filterKey: String,
    filterValue: String,
    onBack: () -> Unit,
    onNavigateToTransactions: (String) -> Unit = {},
) {
    var data by remember { mutableStateOf<Any?>(null) }
    var loading by remember { mutableStateOf(true) }

    LaunchedEffect(filterKey, filterValue) {
        loading = true
        try {
            // Fetch detail data from API
            val client = okhttp3.OkHttpClient()
            val baseUrl = "https://khata.algosculptor.com/api"
            val url = "$baseUrl/txns/analytics/detail?$filterKey=$filterValue"
            val request = okhttp3.Request.Builder().url(url).build()
            val response = client.newCall(request).execute()
            val body = response.body?.string()
            if (body != null) {
                data = com.google.gson.Gson().fromJson(body, com.google.gson.JsonObject::class.java)
            }
        } catch (_: Exception) {}
        loading = false
    }

    Scaffold(
        topBar = {
            TopAppBar(title = { Text("$filterKey: $filterValue", fontSize = 16.sp) },
                navigationIcon = { IconButton(onClick = onBack) { Icon(Icons.Default.ArrowBack, contentDescription = "Back") } })
        }
    ) { innerPadding ->
        LazyColumn(Modifier.fillMaxSize().padding(innerPadding).padding(horizontal = 16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            if (loading) {
                item { Box(Modifier.fillMaxWidth().height(200.dp), contentAlignment = Alignment.Center) { CircularProgressIndicator() } }
                return@LazyColumn
            }

            if (data == null) {
                item { Text("No data", modifier = Modifier.padding(vertical = 24.dp), color = KhataColors.text2) }
                return@LazyColumn
            }

            item { Spacer(Modifier.height(8.dp)) }

            // Totals
            item {
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp), modifier = Modifier.fillMaxWidth()) {
                    KhataCard(Modifier.weight(1f)) { KhataCardBody {
                        Text("Total spent", fontSize = 10.sp, color = KhataColors.textMuted)
                        Text(formatINR((data as com.google.gson.JsonObject).get("total_spent")?.asDouble ?: 0.0), fontSize = 16.sp, fontWeight = FontWeight.Bold, style = LocalTextStyle.current.copy(fontFeatureSettings = "tnum"))
                    } }
                    KhataCard(Modifier.weight(1f)) { KhataCardBody {
                        Text("Total earned", fontSize = 10.sp, color = KhataColors.textMuted)
                        Text(formatINR((data as com.google.gson.JsonObject).get("total_earned")?.asDouble ?: 0.0), fontSize = 16.sp, fontWeight = FontWeight.Bold, style = LocalTextStyle.current.copy(fontFeatureSettings = "tnum"))
                    } }
                }
            }
        }
    }
}
