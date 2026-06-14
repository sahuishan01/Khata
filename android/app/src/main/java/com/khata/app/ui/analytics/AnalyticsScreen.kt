package com.khata.app.ui.analytics

import androidx.compose.foundation.clickable
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.khata.app.api.AnalysisStats
import com.khata.app.api.DashboardStats
import com.khata.app.ui.charts.*
import com.khata.app.util.formatDate
import com.khata.app.ui.components.shared.KhataCard
import com.khata.app.ui.components.shared.KhataCardBody
import com.khata.app.ui.components.shared.KhataCardHeader
import com.khata.app.ui.theme.KhataColors
import com.khata.app.util.formatINR

private data class SectionToggle(
    val key: String, val label: String, var visible: Boolean = true
)

@Composable
fun AnalyticsScreen(
    stats: DashboardStats?,
    analysis: AnalysisStats?,
    isLoading: Boolean,
    onRefresh: () -> Unit,
    onNavigateToDetail: (key: String, value: String) -> Unit = { _, _ -> },
    onNavigateToTransactions: (filter: String) -> Unit = {},
) {
    LaunchedEffect(Unit) { onRefresh() }

    var showCustomize by remember { mutableStateOf(false) }
    var categoryChartType by remember { mutableStateOf("donut") } // "donut" or "list"
    val toggles = remember {
        mutableStateListOf(
            SectionToggle("monthly_bar", "Monthly Trend"),
            SectionToggle("net_line", "Net Worth Trend"),
            SectionToggle("waterfall", "Surplus/Deficit"),
            SectionToggle("category", "Category Breakdown"),
            SectionToggle("comparison", "Month Comparison"),
            SectionToggle("monthly_table", "Monthly Summary"),
            SectionToggle("insights", "Insights"),
        )
    }

    if (showCustomize) {
        AlertDialog(
            onDismissRequest = { showCustomize = false },
            title = { Text("Customize Analytics") },
            text = {
                Column {
                    toggles.forEachIndexed { i, toggle ->
                        Row(
                            modifier = Modifier.fillMaxWidth().padding(vertical = 4.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Checkbox(
                                checked = toggle.visible,
                                onCheckedChange = { toggles[i] = toggle.copy(visible = it) }
                            )
                            Spacer(Modifier.width(8.dp))
                            Text(toggle.label, fontSize = 14.sp)
                        }
                    }
                    Spacer(Modifier.height(12.dp))
                    Text("Category View", fontSize = 13.sp, fontWeight = FontWeight.SemiBold, color = KhataColors.text2)
                    Row {
                        FilterChip(
                            selected = categoryChartType == "donut",
                            onClick = { categoryChartType = "donut" },
                            label = { Text("Donut", fontSize = 12.sp) },
                            modifier = Modifier.padding(end = 8.dp)
                        )
                        FilterChip(
                            selected = categoryChartType == "list",
                            onClick = { categoryChartType = "list" },
                            label = { Text("List", fontSize = 12.sp) }
                        )
                    }
                }
            },
            confirmButton = {
                TextButton(onClick = { showCustomize = false }) { Text("Done") }
            }
        )
    }

    LazyColumn(
        modifier = Modifier.fillMaxSize().padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        item {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Column {
                    Text("Analytics", fontSize = 22.sp, fontWeight = FontWeight.Bold)
                    Text("Insights & summaries", fontSize = 13.sp, color = KhataColors.text2)
                }
                FilledTonalIconButton(onClick = { showCustomize = true }) {
                    Icon(Icons.Default.Tune, contentDescription = "Customize")
                }
            }
        }

        if (isLoading && stats == null) {
            item {
                Box(Modifier.fillMaxWidth().height(200.dp), contentAlignment = Alignment.Center) {
                    CircularProgressIndicator()
                }
            }
            return@LazyColumn
        }

        if (stats == null || analysis == null) {
            item {
                Box(Modifier.fillMaxWidth().height(200.dp), contentAlignment = Alignment.Center) {
                    Text("Upload a statement to see analytics", color = KhataColors.text2)
                }
            }
            return@LazyColumn
        }

        // Monthly bar chart
        if (toggles.find { it.key == "monthly_bar" }?.visible == true) {
            item {
                KhataCard(Modifier.fillMaxWidth()) {
                    KhataCardHeader("MONTHLY TREND")
                    KhataCardBody {
                        MonthlyBarChart(data = stats.monthly)
                    }
                }
            }
        }

        // Net worth line chart
        if (toggles.find { it.key == "net_line" }?.visible == true) {
            item {
                KhataCard(Modifier.fillMaxWidth()) {
                    KhataCardHeader("NET WORTH TREND")
                    KhataCardBody {
                        NetWorthLineChart(data = stats.monthly)
                    }
                }
            }
        }

        // Waterfall
        if (toggles.find { it.key == "waterfall" }?.visible == true) {
            item {
                KhataCard(Modifier.fillMaxWidth()) {
                    KhataCardHeader("SURPLUS / DEFICIT")
                    KhataCardBody {
                        WaterfallChart(data = stats.monthly)
                    }
                }
            }
        }

        // Category breakdown
        if (toggles.find { it.key == "category" }?.visible == true) {
            item {
                KhataCard(Modifier.fillMaxWidth()) {
                    KhataCardHeader("SPENDING BY CATEGORY") {
                        Text(categoryChartType.uppercase(), fontSize = 9.sp, color = KhataColors.text2)
                    }
                    KhataCardBody {
                        if (analysis.categoryBreakdown.isNotEmpty()) {
                            if (categoryChartType == "donut") {
                                CategoryPieChart(data = analysis.categoryBreakdown)
                            } else {
                                analysis.categoryBreakdown.forEach { c ->
                                    Row(Modifier.fillMaxWidth().padding(vertical = 4.dp).clickable { onNavigateToDetail("category", c.category) }, horizontalArrangement = Arrangement.SpaceBetween) {
                                        Row(verticalAlignment = Alignment.CenterVertically) {
                                            val idx = analysis.categoryBreakdown.indexOf(c)
                                            Box(Modifier.size(10.dp).padding(0.dp).clip(androidx.compose.foundation.shape.CircleShape).background(listOf(
                                                Color(0xFF8479F2), Color(0xFF2EC27E), Color(0xFFEE6B4D),
                                                Color(0xFFFDCB6E), Color(0xFF74B9FF), Color(0xFFA29BFE),
                                                Color(0xFF55EFC4), Color(0xFFFF7675), Color(0xFFFD79A8),
                                                Color(0xFF81ECEC), Color(0xFFFAB1A0), Color(0xFF636E72),
                                            )[idx % 12]))
                                            Spacer(Modifier.width(6.dp))
                                            Text(c.category, fontSize = 13.sp, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.weight(1f))
                                        }
                                        Text(formatINR(c.amount), fontSize = 13.sp, fontWeight = FontWeight.SemiBold, style = LocalTextStyle.current.copy(fontFeatureSettings = "tnum"))
                                    }
                                    HorizontalDivider(color = KhataColors.hairline)
                                }
                            }
                        } else {
                            Column(horizontalAlignment = Alignment.CenterHorizontally, modifier = Modifier.fillMaxWidth().padding(vertical = 12.dp)) {
                                Text("📊", fontSize = 24.sp)
                                Spacer(Modifier.height(4.dp))
                                Text("No spending data yet", fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
                                Spacer(Modifier.height(2.dp))
                                Text("Upload more statements to see category breakdowns", fontSize = 12.sp, color = KhataColors.text2)
                            }
                        }
                    }
                }
            }
        }

        // Month comparison
        if (toggles.find { it.key == "comparison" }?.visible == true && analysis.monthComparison.lastMonth > 0) {
            item {
                KhataCard(Modifier.fillMaxWidth().clickable {
                    val now = java.time.LocalDate.now()
                    onNavigateToDetail("month", now.format(java.time.format.DateTimeFormatter.ofPattern("YYYY-MM")))
                }) {
                    KhataCardBody {
                        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
                            Column {
                                Text("This month", fontSize = 12.sp, color = KhataColors.text2)
                                Text(formatINR(analysis.monthComparison.thisMonth), fontSize = 18.sp, fontWeight = FontWeight.Bold, style = LocalTextStyle.current.copy(fontFeatureSettings = "tnum"))
                            }
                            Column(horizontalAlignment = Alignment.End) {
                                Text("vs last month", fontSize = 12.sp, color = KhataColors.text2)
                                Text(
                                    "${if (analysis.monthComparison.changePct > 0) "↑" else "↓"} ${"%.1f".format(kotlin.math.abs(analysis.monthComparison.changePct))}%",
                                    fontSize = 18.sp, fontWeight = FontWeight.Bold,
                                    color = if (analysis.monthComparison.changePct > 0) KhataColors.expense else KhataColors.income
                                )
                            }
                        }
                    }
                }
            }
        }

        // Insights
        if (toggles.find { it.key == "insights" }?.visible == true) {
            item {
                KhataCard(Modifier.fillMaxWidth()) {
                    KhataCardHeader("INSIGHTS")
                    KhataCardBody {
                        Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                            Surface(shape = MaterialTheme.shapes.small, color = KhataColors.brandSoft, modifier = Modifier.clickable {
                                val now = java.time.LocalDate.now()
                                onNavigateToDetail("month", now.format(java.time.format.DateTimeFormatter.ofPattern("YYYY-MM")))
                            }) {
                                Text("Savings rate: ${"%.0f".format(analysis.savingsRatePct)}% of income", fontSize = 13.sp, modifier = Modifier.padding(10.dp))
                            }
                            if (analysis.monthComparison.lastMonth > 0) {
                                val dir = if (analysis.monthComparison.changePct > 0) "up" else "down"
                                Surface(shape = MaterialTheme.shapes.small, color = if (analysis.monthComparison.changePct > 0) KhataColors.expenseSoft else KhataColors.incomeSoft) {
                                    Text("Spending $dir ${"%.0f".format(kotlin.math.abs(analysis.monthComparison.changePct))}% vs last month", fontSize = 13.sp, modifier = Modifier.padding(10.dp))
                                }
                            }
                        }
                    }
                }
            }
        }

        // Monthly summary
        if (toggles.find { it.key == "monthly_table" }?.visible == true && stats.monthly.isNotEmpty()) {
            item {
                KhataCard(Modifier.fillMaxWidth()) {
                    KhataCardHeader("MONTHLY SUMMARY")
                    KhataCardBody {
                        stats.monthly.reversed().forEach { month ->
                            val net = month.earned - month.spent
                            Row(Modifier.fillMaxWidth().padding(vertical = 5.dp).clickable { onNavigateToDetail("month", month.month) }, verticalAlignment = Alignment.CenterVertically) {
                                Column(Modifier.weight(1f)) { Text(month.month, fontSize = 13.sp, fontWeight = FontWeight.Medium) }
                                Column(horizontalAlignment = Alignment.End) {
                                    Text(formatINR(month.spent), fontSize = 11.sp, color = KhataColors.expense, style = LocalTextStyle.current.copy(fontFeatureSettings = "tnum"))
                                    Text(formatINR(month.earned), fontSize = 11.sp, color = KhataColors.income, style = LocalTextStyle.current.copy(fontFeatureSettings = "tnum"))
                                }
                                Spacer(Modifier.width(10.dp))
                                Text(formatINR(net, sign = true), fontSize = 13.sp, fontWeight = FontWeight.SemiBold,
                                    color = if (net >= 0) KhataColors.income else KhataColors.expense,
                                    style = LocalTextStyle.current.copy(fontFeatureSettings = "tnum"))
                            }
                            HorizontalDivider(color = KhataColors.hairline)
                        }
                    }
                }
            }
        }

        item { Spacer(Modifier.height(16.dp)) }
    }
}
