package com.khata.app.ui.analytics

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.khata.app.api.DashboardStats
import com.khata.app.api.AnalysisStats
import com.khata.app.ui.charts.CategoryPieChart
import com.khata.app.ui.charts.MonthlyBarChart

private fun fmt(n: Double) = "₹${String.format("%,.0f", n)}"
private fun fmtDec(n: Double) = "₹${String.format("%,.2f", n)}"

@Composable
fun AnalyticsScreen(
    stats: DashboardStats?,
    analysis: AnalysisStats?,
    isLoading: Boolean,
    onRefresh: () -> Unit
) {
    LaunchedEffect(Unit) { onRefresh() }

    LazyColumn(
        modifier = Modifier.fillMaxSize().padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        item {
            Text("Analytics", fontSize = 22.sp, fontWeight = FontWeight.Bold)
            Text("Insights & summaries", fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }

        if (isLoading && stats == null) {
            item {
                Box(modifier = Modifier.fillMaxWidth().height(200.dp), contentAlignment = Alignment.Center) {
                    CircularProgressIndicator()
                }
            }
            return@LazyColumn
        }

        if (stats == null || analysis == null) {
            item {
                Box(modifier = Modifier.fillMaxWidth().height(200.dp), contentAlignment = Alignment.Center) {
                    Text("Upload a statement to see analytics", color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }
            return@LazyColumn
        }

        // Monthly chart
        item {
            Card(modifier = Modifier.fillMaxWidth(), shape = RoundedCornerShape(14.dp)) {
                Column(modifier = Modifier.padding(16.dp)) {
                    Text("MONTHLY TREND", fontSize = 11.sp, fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f),
                        letterSpacing = 0.8.sp)
                    Spacer(Modifier.height(12.dp))
                    MonthlyBarChart(data = stats.monthly)
                }
            }
        }

        // Category breakdown
        item {
            Card(modifier = Modifier.fillMaxWidth(), shape = RoundedCornerShape(14.dp)) {
                Column(modifier = Modifier.padding(16.dp)) {
                    Text("SPENDING BY CATEGORY", fontSize = 11.sp, fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f),
                        letterSpacing = 0.8.sp)
                    Spacer(Modifier.height(12.dp))
                    if (analysis.categoryBreakdown.isNotEmpty()) {
                        CategoryPieChart(data = analysis.categoryBreakdown)
                    } else {
                        Text("No spending data yet", color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                }
            }
        }

        // Month comparison
        if (analysis.monthComparison.lastMonth > 0) {
            item {
                Card(modifier = Modifier.fillMaxWidth(), shape = RoundedCornerShape(14.dp)) {
                    Row(
                        modifier = Modifier.fillMaxWidth().padding(16.dp),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Column {
                            Text("This month", fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            Text(fmt(analysis.monthComparison.thisMonth), fontSize = 18.sp, fontWeight = FontWeight.Bold)
                        }
                        Column(horizontalAlignment = Alignment.End) {
                            Text("vs last month", fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            Text(
                                "${if (analysis.monthComparison.changePct > 0) "↑" else "↓"} ${"%.1f".format(kotlin.math.abs(analysis.monthComparison.changePct))}%",
                                fontSize = 18.sp, fontWeight = FontWeight.Bold,
                                color = if (analysis.monthComparison.changePct > 0)
                                    MaterialTheme.colorScheme.error
                                else
                                    MaterialTheme.colorScheme.secondary
                            )
                        }
                    }
                }
            }
        }

        // Summary cards
        item {
            Row(horizontalArrangement = Arrangement.spacedBy(10.dp), modifier = Modifier.fillMaxWidth()) {
                Card(modifier = Modifier.weight(1f), shape = RoundedCornerShape(14.dp)) {
                    Column(modifier = Modifier.padding(14.dp)) {
                        Text("AVG DAILY", fontSize = 10.sp, fontWeight = FontWeight.Bold,
                            color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f))
                        Text(fmt(analysis.avgDailySpend), fontSize = 18.sp, fontWeight = FontWeight.Bold)
                    }
                }
                Card(modifier = Modifier.weight(1f), shape = RoundedCornerShape(14.dp)) {
                    Column(modifier = Modifier.padding(14.dp)) {
                        Text("SAVINGS RATE", fontSize = 10.sp, fontWeight = FontWeight.Bold,
                            color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f))
                        Text("${"%.1f".format(analysis.savingsRatePct)}%", fontSize = 18.sp, fontWeight = FontWeight.Bold,
                            color = if (analysis.savingsRatePct >= 20) MaterialTheme.colorScheme.secondary
                                    else MaterialTheme.colorScheme.error)
                    }
                }
                Card(modifier = Modifier.weight(1f), shape = RoundedCornerShape(14.dp)) {
                    Column(modifier = Modifier.padding(14.dp)) {
                        Text("TOTAL TXNS", fontSize = 10.sp, fontWeight = FontWeight.Bold,
                            color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f))
                        Text("${analysis.totalTransactions}", fontSize = 18.sp, fontWeight = FontWeight.Bold)
                    }
                }
            }
        }

        // Largest expense
        if (analysis.largestExpense != null) {
            item {
                Card(modifier = Modifier.fillMaxWidth(), shape = RoundedCornerShape(14.dp)) {
                    Column(modifier = Modifier.padding(16.dp)) {
                        Text("LARGEST EXPENSE", fontSize = 11.sp, fontWeight = FontWeight.Bold,
                            color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f),
                            letterSpacing = 0.8.sp)
                        Spacer(Modifier.height(8.dp))
                        Text(fmtDec(analysis.largestExpense!!.amount), fontSize = 24.sp,
                            fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.error)
                        Text(analysis.largestExpense!!.description, fontSize = 13.sp,
                            maxLines = 1, overflow = TextOverflow.Ellipsis)
                        Text(analysis.largestExpense!!.valueDate, fontSize = 12.sp,
                            color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                }
            }
        }

        // Transaction by month (grouped summary)
        if (stats.monthly.isNotEmpty()) {
            item {
                Card(modifier = Modifier.fillMaxWidth(), shape = RoundedCornerShape(14.dp)) {
                    Column(modifier = Modifier.padding(16.dp)) {
                        Text("MONTHLY SUMMARY", fontSize = 11.sp, fontWeight = FontWeight.Bold,
                            color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f),
                            letterSpacing = 0.8.sp)
                        Spacer(Modifier.height(8.dp))

                        stats.monthly.reversed().forEach { month ->
                            val net = month.earned - month.spent
                            Row(
                                modifier = Modifier.fillMaxWidth().padding(vertical = 6.dp),
                                verticalAlignment = Alignment.CenterVertically
                            ) {
                                Column(modifier = Modifier.weight(1f)) {
                                    Text(month.month, fontSize = 13.sp, fontWeight = FontWeight.Medium)
                                }
                                Column(horizontalAlignment = Alignment.End) {
                                    Text(fmt(month.spent), fontSize = 12.sp, color = MaterialTheme.colorScheme.error)
                                    Text(fmt(month.earned), fontSize = 12.sp, color = MaterialTheme.colorScheme.secondary)
                                }
                                Spacer(Modifier.width(12.dp))
                                Text(
                                    if (net >= 0) "+${fmt(net)}" else fmt(net),
                                    fontSize = 13.sp, fontWeight = FontWeight.SemiBold,
                                    color = if (net >= 0) MaterialTheme.colorScheme.secondary
                                            else MaterialTheme.colorScheme.error
                                )
                            }
                            HorizontalDivider(color = MaterialTheme.colorScheme.outline.copy(alpha = 0.3f))
                        }
                    }
                }
            }
        }

        // Top debits
        item {
            Card(modifier = Modifier.fillMaxWidth(), shape = RoundedCornerShape(14.dp)) {
                Column(modifier = Modifier.padding(16.dp)) {
                    Text("TOP SPENDING", fontSize = 11.sp, fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f),
                        letterSpacing = 0.8.sp)
                    Spacer(Modifier.height(8.dp))
                    stats.topDebits.take(7).forEach { t ->
                        Row(
                            modifier = Modifier.fillMaxWidth().padding(vertical = 4.dp),
                            horizontalArrangement = Arrangement.SpaceBetween
                        ) {
                            Text(t.description, fontSize = 13.sp, maxLines = 1,
                                overflow = TextOverflow.Ellipsis, modifier = Modifier.weight(1f))
                            Text(fmtDec(t.total), fontSize = 13.sp, fontWeight = FontWeight.SemiBold,
                                color = MaterialTheme.colorScheme.error)
                        }
                        HorizontalDivider(color = MaterialTheme.colorScheme.outline.copy(alpha = 0.3f))
                    }
                }
            }
        }

        item { Spacer(Modifier.height(16.dp)) }
    }
}
