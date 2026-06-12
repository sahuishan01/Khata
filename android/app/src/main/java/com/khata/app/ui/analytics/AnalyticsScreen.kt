package com.khata.app.ui.analytics

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.khata.app.api.AnalysisStats
import com.khata.app.api.DashboardStats
import com.khata.app.ui.charts.*

private fun fmt(n: Double) = "₹${String.format("%,.0f", n)}"
private fun fmtDec(n: Double) = "₹${String.format("%,.2f", n)}"

private data class SectionToggle(
    val key: String, val label: String, var visible: Boolean = true
)

@Composable
fun AnalyticsScreen(
    stats: DashboardStats?,
    analysis: AnalysisStats?,
    isLoading: Boolean,
    onRefresh: () -> Unit
) {
    LaunchedEffect(Unit) { onRefresh() }

    var showCustomize by remember { mutableStateOf(false) }
    var categoryChartType by remember { mutableStateOf("donut") } // "donut" or "list"
    val toggles = remember {
        mutableStateListOf(
            SectionToggle("monthly_bar", "Monthly Bar Chart"),
            SectionToggle("net_line", "Net Worth Line Chart"),
            SectionToggle("waterfall", "Surplus/Deficit Waterfall"),
            SectionToggle("category", "Category Breakdown"),
            SectionToggle("comparison", "Month Comparison"),
            SectionToggle("summary_cards", "Summary Cards"),
            SectionToggle("largest", "Largest Expense"),
            SectionToggle("monthly_table", "Monthly Table"),
            SectionToggle("top_debits", "Top Spending"),
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
                    Text("Category View", fontSize = 13.sp, fontWeight = FontWeight.SemiBold, color = MaterialTheme.colorScheme.onSurfaceVariant)
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
                    Text("Insights & summaries", fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
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
                    Text("Upload a statement to see analytics", color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }
            return@LazyColumn
        }

        // Monthly bar chart
        if (toggles.find { it.key == "monthly_bar" }?.visible == true) {
            item {
                Card(Modifier.fillMaxWidth(), shape = RoundedCornerShape(14.dp)) {
                    Column(Modifier.padding(16.dp)) {
                        Text("MONTHLY TREND", fontSize = 11.sp, fontWeight = FontWeight.Bold,
                            color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f), letterSpacing = 0.8.sp)
                        Spacer(Modifier.height(12.dp))
                        MonthlyBarChart(data = stats.monthly)
                    }
                }
            }
        }

        // Net worth line chart
        if (toggles.find { it.key == "net_line" }?.visible == true) {
            item {
                Card(Modifier.fillMaxWidth(), shape = RoundedCornerShape(14.dp)) {
                    Column(Modifier.padding(16.dp)) {
                        Text("NET WORTH TREND", fontSize = 11.sp, fontWeight = FontWeight.Bold,
                            color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f), letterSpacing = 0.8.sp)
                        Spacer(Modifier.height(8.dp))
                        NetWorthLineChart(data = stats.monthly)
                    }
                }
            }
        }

        // Waterfall
        if (toggles.find { it.key == "waterfall" }?.visible == true) {
            item {
                Card(Modifier.fillMaxWidth(), shape = RoundedCornerShape(14.dp)) {
                    Column(Modifier.padding(16.dp)) {
                        Text("SURPLUS / DEFICIT", fontSize = 11.sp, fontWeight = FontWeight.Bold,
                            color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f), letterSpacing = 0.8.sp)
                        Spacer(Modifier.height(8.dp))
                        WaterfallChart(data = stats.monthly)
                    }
                }
            }
        }

        // Category breakdown
        if (toggles.find { it.key == "category" }?.visible == true) {
            item {
                Card(Modifier.fillMaxWidth(), shape = RoundedCornerShape(14.dp)) {
                    Column(Modifier.padding(16.dp)) {
                        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
                            Text("SPENDING BY CATEGORY", fontSize = 11.sp, fontWeight = FontWeight.Bold,
                                color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f), letterSpacing = 0.8.sp)
                            Text(categoryChartType.uppercase(), fontSize = 9.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                        Spacer(Modifier.height(12.dp))
                        if (analysis.categoryBreakdown.isNotEmpty()) {
                            if (categoryChartType == "donut") {
                                CategoryPieChart(data = analysis.categoryBreakdown)
                            } else {
                                analysis.categoryBreakdown.forEach { c ->
                                    Row(Modifier.fillMaxWidth().padding(vertical = 4.dp), horizontalArrangement = Arrangement.SpaceBetween) {
                                        Row(verticalAlignment = Alignment.CenterVertically) {
                                            val idx = analysis.categoryBreakdown.indexOf(c)
                                            Box(Modifier.size(10.dp).padding(0.dp).clip(androidx.compose.foundation.shape.CircleShape).background(listOf(
                                                Color(0xFF6C5CE7), Color(0xFF00B894), Color(0xFFE17055),
                                                Color(0xFFFDCB6E), Color(0xFF74B9FF), Color(0xFFA29BFE),
                                                Color(0xFF55EFC4), Color(0xFFFF7675), Color(0xFFFD79A8),
                                                Color(0xFF81ECEC), Color(0xFFFAB1A0), Color(0xFF636E72),
                                            )[idx % 12]))
                                            Spacer(Modifier.width(6.dp))
                                            Text(c.category, fontSize = 13.sp, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.weight(1f))
                                        }
                                        Text("₹${String.format("%,.0f", c.amount)}", fontSize = 13.sp, fontWeight = FontWeight.SemiBold)
                                    }
                                    HorizontalDivider(color = MaterialTheme.colorScheme.outline.copy(alpha = 0.3f))
                                }
                            }
                        } else {
                            Text("No spending data yet", color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                    }
                }
            }
        }

        // Month comparison
        if (toggles.find { it.key == "comparison" }?.visible == true && analysis.monthComparison.lastMonth > 0) {
            item {
                Card(Modifier.fillMaxWidth(), shape = RoundedCornerShape(14.dp)) {
                    Row(Modifier.fillMaxWidth().padding(16.dp), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
                        Column {
                            Text("This month", fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            Text(fmt(analysis.monthComparison.thisMonth), fontSize = 18.sp, fontWeight = FontWeight.Bold)
                        }
                        Column(horizontalAlignment = Alignment.End) {
                            Text("vs last month", fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            Text(
                                "${if (analysis.monthComparison.changePct > 0) "↑" else "↓"} ${"%.1f".format(kotlin.math.abs(analysis.monthComparison.changePct))}%",
                                fontSize = 18.sp, fontWeight = FontWeight.Bold,
                                color = if (analysis.monthComparison.changePct > 0) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.secondary
                            )
                        }
                    }
                }
            }
        }

        // Summary cards
        if (toggles.find { it.key == "summary_cards" }?.visible == true) {
            item {
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp), modifier = Modifier.fillMaxWidth()) {
                    Card(Modifier.weight(1f), shape = RoundedCornerShape(14.dp)) {
                        Column(Modifier.padding(12.dp)) {
                            Text("AVG DAILY", fontSize = 9.sp, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f))
                            Text(fmt(analysis.avgDailySpend), fontSize = 16.sp, fontWeight = FontWeight.Bold)
                        }
                    }
                    Card(Modifier.weight(1f), shape = RoundedCornerShape(14.dp)) {
                        Column(Modifier.padding(12.dp)) {
                            Text("SAVINGS", fontSize = 9.sp, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f))
                            Text("${"%.1f".format(analysis.savingsRatePct)}%", fontSize = 16.sp, fontWeight = FontWeight.Bold,
                                color = if (analysis.savingsRatePct >= 20) MaterialTheme.colorScheme.secondary else MaterialTheme.colorScheme.error)
                        }
                    }
                    Card(Modifier.weight(1f), shape = RoundedCornerShape(14.dp)) {
                        Column(Modifier.padding(12.dp)) {
                            Text("TXNS", fontSize = 9.sp, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f))
                            Text("${analysis.totalTransactions}", fontSize = 16.sp, fontWeight = FontWeight.Bold)
                        }
                    }
                    Card(Modifier.weight(1f), shape = RoundedCornerShape(14.dp)) {
                        Column(Modifier.padding(12.dp)) {
                            Text("NET", fontSize = 9.sp, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f))
                            val net = stats.totalEarned - stats.totalSpent
                            Text(fmt(net), fontSize = 16.sp, fontWeight = FontWeight.Bold,
                                color = if (net >= 0) MaterialTheme.colorScheme.secondary else MaterialTheme.colorScheme.error)
                        }
                    }
                }
            }
        }

        // Largest expense
        if (toggles.find { it.key == "largest" }?.visible == true && analysis.largestExpense != null) {
            item {
                Card(Modifier.fillMaxWidth(), shape = RoundedCornerShape(14.dp)) {
                    Column(Modifier.padding(16.dp)) {
                        Text("LARGEST EXPENSE", fontSize = 11.sp, fontWeight = FontWeight.Bold,
                            color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f), letterSpacing = 0.8.sp)
                        Spacer(Modifier.height(8.dp))
                        Text(fmtDec(analysis.largestExpense!!.amount), fontSize = 24.sp, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.error)
                        Text(analysis.largestExpense!!.description, fontSize = 13.sp, maxLines = 1, overflow = TextOverflow.Ellipsis)
                        Text(analysis.largestExpense!!.valueDate, fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                }
            }
        }

        // Monthly table
        if (toggles.find { it.key == "monthly_table" }?.visible == true && stats.monthly.isNotEmpty()) {
            item {
                Card(Modifier.fillMaxWidth(), shape = RoundedCornerShape(14.dp)) {
                    Column(Modifier.padding(16.dp)) {
                        Text("MONTHLY SUMMARY", fontSize = 11.sp, fontWeight = FontWeight.Bold,
                            color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f), letterSpacing = 0.8.sp)
                        Spacer(Modifier.height(8.dp))
                        stats.monthly.reversed().forEach { month ->
                            val net = month.earned - month.spent
                            Row(Modifier.fillMaxWidth().padding(vertical = 5.dp), verticalAlignment = Alignment.CenterVertically) {
                                Column(Modifier.weight(1f)) { Text(month.month, fontSize = 13.sp, fontWeight = FontWeight.Medium) }
                                Column(horizontalAlignment = Alignment.End) {
                                    Text(fmt(month.spent), fontSize = 11.sp, color = MaterialTheme.colorScheme.error)
                                    Text(fmt(month.earned), fontSize = 11.sp, color = MaterialTheme.colorScheme.secondary)
                                }
                                Spacer(Modifier.width(10.dp))
                                Text(if (net >= 0) "+${fmt(net)}" else fmt(net), fontSize = 13.sp, fontWeight = FontWeight.SemiBold,
                                    color = if (net >= 0) MaterialTheme.colorScheme.secondary else MaterialTheme.colorScheme.error)
                            }
                            HorizontalDivider(color = MaterialTheme.colorScheme.outline.copy(alpha = 0.3f))
                        }
                    }
                }
            }
        }

        // Top debits
        if (toggles.find { it.key == "top_debits" }?.visible == true) {
            item {
                Card(Modifier.fillMaxWidth(), shape = RoundedCornerShape(14.dp)) {
                    Column(Modifier.padding(16.dp)) {
                        Text("TOP SPENDING", fontSize = 11.sp, fontWeight = FontWeight.Bold,
                            color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f), letterSpacing = 0.8.sp)
                        Spacer(Modifier.height(8.dp))
                        stats.topDebits.take(7).forEach { t ->
                            Row(Modifier.fillMaxWidth().padding(vertical = 4.dp), horizontalArrangement = Arrangement.SpaceBetween) {
                                Text(t.description, fontSize = 13.sp, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.weight(1f))
                                Text(fmtDec(t.total), fontSize = 13.sp, fontWeight = FontWeight.SemiBold, color = MaterialTheme.colorScheme.error)
                            }
                            HorizontalDivider(color = MaterialTheme.colorScheme.outline.copy(alpha = 0.3f))
                        }
                    }
                }
            }
        }

        item { Spacer(Modifier.height(16.dp)) }
    }
}
