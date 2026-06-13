package com.khata.app.ui.dashboard

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
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
import com.khata.app.api.AnalysisStats
import com.khata.app.api.DashboardStats
import com.khata.app.ui.charts.CategoryPieChart
import com.khata.app.ui.charts.MonthlyBarChart
import com.khata.app.ui.components.StatCard
import com.khata.app.util.formatDate
import com.khata.app.util.formatINR
import com.khata.app.util.maskDescription

@Composable
fun DashboardScreen(
    stats: DashboardStats?,
    analysis: AnalysisStats?,
    isLoading: Boolean,
    error: String?,
    blurMode: Boolean = true,
    onRefresh: () -> Unit,
    onNavigateToTransactions: (String) -> Unit = {}
) {
    LaunchedEffect(Unit) { onRefresh() }

    LazyColumn(
        modifier = Modifier.fillMaxSize().padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        item {
            Text("Dashboard", fontSize = 22.sp, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.onBackground)
            Text("Your financial overview", fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }

        if (error != null) {
            item {
                Card(
                    colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.errorContainer)
                ) {
                    Text(error, modifier = Modifier.padding(12.dp), fontSize = 13.sp)
                }
            }
        }

        if (stats == null && isLoading) {
            item {
                Box(modifier = Modifier.fillMaxWidth().height(200.dp), contentAlignment = Alignment.Center) {
                    CircularProgressIndicator()
                }
            }
        }

        if (stats != null) {
            item {
                Row(horizontalArrangement = Arrangement.spacedBy(10.dp), modifier = Modifier.fillMaxWidth()) {
                    StatCard(
                        label = "Net Balance",
                        value = formatINR(stats.net),
                        icon = { Icon(Icons.Default.AccountBalance, contentDescription = null, modifier = Modifier.size(18.dp)) },
                        accentColor = MaterialTheme.colorScheme.primary,
                        modifier = Modifier.weight(1f)
                    )
                    StatCard(
                        label = "Total Spent",
                        value = formatINR(stats.totalSpent),
                        icon = { Icon(Icons.Default.TrendingDown, contentDescription = null, modifier = Modifier.size(18.dp)) },
                        accentColor = MaterialTheme.colorScheme.error,
                        modifier = Modifier.weight(1f)
                    )
                }
            }

            item {
                Row(horizontalArrangement = Arrangement.spacedBy(10.dp), modifier = Modifier.fillMaxWidth()) {
                    StatCard(
                        label = "Total Earned",
                        value = formatINR(stats.totalEarned),
                        icon = { Icon(Icons.Default.TrendingUp, contentDescription = null, modifier = Modifier.size(18.dp)) },
                        accentColor = MaterialTheme.colorScheme.secondary,
                        subtitle = "${analysis?.totalTransactions ?: 0} transactions",
                        modifier = Modifier.weight(1f)
                    )
                    StatCard(
                        label = "Savings Rate",
                        value = "${"%.1f".format(analysis?.savingsRatePct ?: 0.0)}%",
                        icon = { Icon(Icons.Default.Savings, contentDescription = null, modifier = Modifier.size(18.dp)) },
                        accentColor = if ((analysis?.savingsRatePct ?: 0.0) >= 20) MaterialTheme.colorScheme.secondary
                            else MaterialTheme.colorScheme.tertiary,
                        modifier = Modifier.weight(1f)
                    )
                }
            }

            // Invested row
            if ((analysis?.totalInvested ?: 0.0) > 0) {
                item {
                    Row(horizontalArrangement = Arrangement.spacedBy(10.dp), modifier = Modifier.fillMaxWidth()) {
                        StatCard(
                            label = "Invested",
                            value = formatINR(analysis!!.totalInvested),
                            icon = { Icon(Icons.Default.TrendingUp, contentDescription = null, modifier = Modifier.size(18.dp)) },
                            accentColor = MaterialTheme.colorScheme.primary,
                            modifier = Modifier.weight(1f)
                        )
                        Spacer(Modifier.weight(1f))
                    }
                }
            }

            if (analysis != null && analysis.monthComparison.lastMonth > 0) {
                item {
                    Card(
                        modifier = Modifier.fillMaxWidth(),
                        shape = RoundedCornerShape(14.dp),
                        colors = CardDefaults.cardColors(
                            containerColor = if (analysis.monthComparison.changePct > 0)
                                MaterialTheme.colorScheme.errorContainer.copy(alpha = 0.3f)
                            else
                                MaterialTheme.colorScheme.secondaryContainer.copy(alpha = 0.3f)
                        )
                    ) {
                        Row(
                            modifier = Modifier.fillMaxWidth().padding(14.dp),
                            horizontalArrangement = Arrangement.SpaceBetween,
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Column {
                                Text("This month", fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                                Text(formatINR(analysis.monthComparison.thisMonth), fontSize = 16.sp, fontWeight = FontWeight.Bold)
                            }
                            Column(horizontalAlignment = Alignment.End) {
                                Text("vs last month", fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                                Text(
                                    "${if (analysis.monthComparison.changePct > 0) "↑" else "↓"} ${"%.1f".format(kotlin.math.abs(analysis.monthComparison.changePct))}%",
                                    fontSize = 16.sp,
                                    fontWeight = FontWeight.Bold,
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

            // Monthly bar chart
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

            // Category pie chart
            if (analysis != null && analysis.categoryBreakdown.isNotEmpty()) {
                item {
                    Card(modifier = Modifier.fillMaxWidth(), shape = RoundedCornerShape(14.dp)) {
                        Column(modifier = Modifier.padding(16.dp)) {
                            Text("SPENDING BY CATEGORY", fontSize = 11.sp, fontWeight = FontWeight.Bold,
                                color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f),
                                letterSpacing = 0.8.sp)
                            Spacer(Modifier.height(12.dp))
                            CategoryPieChart(data = analysis.categoryBreakdown, onCategoryClick = { cat -> onNavigateToTransactions(cat) })
                        }
                    }
                }
            }

            if (analysis?.largestExpense != null) {
                item {
                    Card(
                        modifier = Modifier.fillMaxWidth(),
                        shape = RoundedCornerShape(14.dp)
                    ) {
                        Column(modifier = Modifier.padding(16.dp)) {
                            Text("LARGEST EXPENSE", fontSize = 11.sp, fontWeight = FontWeight.Bold,
                                color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f),
                                letterSpacing = 0.8.sp)
                            Spacer(Modifier.height(8.dp))
                            Text(formatINR(analysis.largestExpense!!.amount), fontSize = 26.sp,
                                fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.error)
                            Text(maskDescription(analysis.largestExpense!!.description, blurMode), fontSize = 13.sp,
                                maxLines = 1, overflow = TextOverflow.Ellipsis,
                                color = MaterialTheme.colorScheme.onBackground)
                            Text(formatDate(analysis.largestExpense!!.valueDate), fontSize = 12.sp,
                                color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                    }
                }
            }

            item {
                Card(
                    modifier = Modifier.fillMaxWidth(),
                    shape = RoundedCornerShape(14.dp)
                ) {
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
                                Text(formatINR(t.total), fontSize = 13.sp, fontWeight = FontWeight.SemiBold,
                                    color = MaterialTheme.colorScheme.error)
                            }
                            HorizontalDivider(
                                modifier = Modifier.padding(vertical = 2.dp),
                                color = MaterialTheme.colorScheme.outline.copy(alpha = 0.3f)
                            )
                        }
                    }
                }
            }
        }
    }
}
