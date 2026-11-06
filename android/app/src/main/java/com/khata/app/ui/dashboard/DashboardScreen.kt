package com.khata.app.ui.dashboard

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.TrendingDown
import androidx.compose.material.icons.automirrored.filled.TrendingUp
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.khata.app.api.AnalysisStats
import com.khata.app.api.DashboardStats
import com.khata.app.ui.charts.CategoryPieChart
import com.khata.app.ui.charts.MonthlyBarChart
import com.khata.app.ui.components.ShimmerCard
import com.khata.app.ui.components.shared.*
import com.khata.app.ui.theme.FigureStyle
import com.khata.app.ui.theme.KhataColors
import com.khata.app.ui.theme.KhataRadius
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
        modifier = Modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        item {
            Text(
                "Dashboard",
                style = MaterialTheme.typography.displaySmall,
                color = KhataColors.text,
                modifier = Modifier.semantics { heading() }.padding(top = 8.dp, bottom = 4.dp),
            )
        }

        if (error != null) {
            item {
                KhataCard(Modifier.fillMaxWidth()) {
                    Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        Text("Couldn't load your dashboard", style = MaterialTheme.typography.titleSmall, color = KhataColors.text)
                        Text(error, style = MaterialTheme.typography.bodyMedium, color = KhataColors.text2)
                        KhataButton(onClick = onRefresh, variant = ButtonVariant.Secondary) { Text("Retry") }
                    }
                }
            }
        }

        if (stats == null && isLoading) {
            item { ShimmerCard(modifier = Modifier.fillMaxWidth(), lines = 4) }
            item { ShimmerCard(modifier = Modifier.fillMaxWidth(), lines = 3) }
        }

        if (stats != null) {
            val hasData = stats.totalSpent > 0 || stats.totalEarned > 0
            if (!hasData) {
                item {
                    KhataEmptyState(
                        emoji = "₹",
                        title = "Your khata is empty",
                        description = "Add a bank or credit-card statement from the Add tab. Khata sorts the transactions into categories and keeps card bill payments out of your spending.",
                    )
                }
                return@LazyColumn
            }

            item { Statement(stats, analysis) }

            if (analysis != null) {
                item { ThisMonth(analysis) }
            }

            item {
                KhataCard(Modifier.fillMaxWidth()) {
                    KhataCardHeader("Earned and spent by month")
                    KhataCardBody { MonthlyBarChart(data = stats.monthly) }
                }
            }

            if (analysis != null && analysis.categoryBreakdown.isNotEmpty()) {
                item {
                    KhataCard(Modifier.fillMaxWidth()) {
                        KhataCardHeader("Where it went", subtitle = "Tap a category to see its transactions")
                        KhataCardBody {
                            CategoryPieChart(data = analysis.categoryBreakdown, onCategoryClick = { cat -> onNavigateToTransactions(cat) })
                        }
                    }
                }
            }

            item {
                KhataCard(Modifier.fillMaxWidth()) {
                    KhataCardHeader("Paid the most")
                    Spacer(Modifier.height(4.dp))
                    val top = stats.topDebits.take(7)
                    top.forEachIndexed { i, t ->
                        KhataListRow(
                            content = { KhataListRowText(primary = maskDescription(t.description, blurMode)) },
                            trailing = { KhataAmount(t.total, size = 14.sp) },
                            divider = i < top.lastIndex,
                        )
                    }
                }
            }

            if (analysis?.largestExpense != null) {
                val le = analysis.largestExpense!!
                item {
                    KhataCard(Modifier.fillMaxWidth()) {
                        KhataCardHeader("Largest single expense")
                        KhataCardBody {
                            KhataAmount(le.amount, size = 24.sp)
                            Column { KhataListRowText(primary = maskDescription(le.description, blurMode), secondary = formatDate(le.valueDate)) }
                        }
                    }
                }
            }
        }
    }
}

/**
 * The statement: the period's books set like a ledger page. A single rule
 * closes the lines; the double rule marks the final figure. Mirrors the web
 * dashboard's `.statement`.
 */
@Composable
private fun Statement(stats: DashboardStats, analysis: AnalysisStats?) {
    val invested = analysis?.totalInvested ?: stats.totalInvested
    Column(
        Modifier
            .fillMaxWidth()
            .clip(KhataRadius.lg)
            .background(KhataColors.surface)
            .border(1.dp, KhataColors.hairline, KhataRadius.lg)
            .drawBehind {
                // the ledger's red margin rule
                val x = 14.dp.toPx()
                drawLine(KhataColors.expense.copy(alpha = 0.5f), Offset(x, 0f), Offset(x, size.height), strokeWidth = 1.dp.toPx())
            }
            .padding(start = 32.dp, end = 20.dp, top = 20.dp, bottom = 20.dp)
    ) {
        Text("All time", style = MaterialTheme.typography.bodyMedium, color = KhataColors.text2)
        Spacer(Modifier.height(8.dp))
        LedgerLine("Earned") { KhataAmount(stats.totalEarned, size = 18.sp, tone = AmountTone.Income) }
        LedgerLine("Spent") { KhataAmount(-stats.totalSpent, size = 18.sp, tone = AmountTone.Expense) }
        HorizontalDivider(color = KhataColors.rule, thickness = 1.dp, modifier = Modifier.padding(top = 4.dp))
        Row(
            Modifier.fillMaxWidth().padding(top = 12.dp, bottom = 12.dp),
            verticalAlignment = Alignment.Bottom,
            horizontalArrangement = Arrangement.SpaceBetween,
        ) {
            Text(
                if (stats.net >= 0) "Saved" else "Overspent",
                style = MaterialTheme.typography.titleMedium, color = KhataColors.text,
            )
            KhataAmount(stats.net, size = 32.sp, tone = if (stats.net >= 0) AmountTone.Neutral else AmountTone.Expense)
        }
        // double rule = final total
        HorizontalDivider(color = KhataColors.rule, thickness = 1.dp)
        Spacer(Modifier.height(2.dp))
        HorizontalDivider(color = KhataColors.rule, thickness = 1.dp)
        Spacer(Modifier.height(16.dp))

        val note = buildString {
            if (analysis != null) append("You kept ${"%.1f".format(analysis.savingsRatePct)}% of what you earned. ")
            if (invested > 0) append("${formatINR(invested)} went to investments and isn't counted as spend. ")
            append("Transfers between your own accounts are left out.")
        }
        Text(
            note,
            style = MaterialTheme.typography.bodyMedium,
            color = if (analysis != null && analysis.savingsRatePct < 20) KhataColors.warn else KhataColors.text2,
        )
    }
}

@Composable
private fun LedgerLine(label: String, value: @Composable () -> Unit) {
    Row(
        Modifier.fillMaxWidth().padding(vertical = 8.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.SpaceBetween,
    ) {
        Text(label, style = MaterialTheme.typography.bodyLarge, color = KhataColors.text2)
        value()
    }
}

@Composable
private fun ThisMonth(analysis: AnalysisStats) {
    val mc = analysis.monthComparison
    val up = mc.changePct > 0
    KhataCard(Modifier.fillMaxWidth()) {
        Column(Modifier.padding(20.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
            Column {
                Text("Spent this month", style = MaterialTheme.typography.bodyMedium, color = KhataColors.text2)
                Spacer(Modifier.height(4.dp))
                KhataAmount(mc.thisMonth, size = 22.sp)
                if (mc.lastMonth > 0) {
                    Spacer(Modifier.height(8.dp))
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Icon(
                            if (up) Icons.AutoMirrored.Filled.TrendingUp else Icons.AutoMirrored.Filled.TrendingDown,
                            contentDescription = null,
                            tint = if (up) KhataColors.expense else KhataColors.income,
                            modifier = Modifier.size(16.dp),
                        )
                        Spacer(Modifier.width(6.dp))
                        Text(
                            "${"%.1f".format(kotlin.math.abs(mc.changePct))}% ${if (up) "more" else "less"} than last month (${formatINR(mc.lastMonth)})",
                            style = MaterialTheme.typography.bodySmall,
                            color = if (up) KhataColors.expense else KhataColors.income,
                        )
                    }
                }
            }
            if (mc.lastMonth > 0) {
                val max = maxOf(mc.thisMonth, mc.lastMonth).coerceAtLeast(1.0)
                Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    CompareBar("This month", (mc.thisMonth / max).toFloat(), KhataColors.text2)
                    CompareBar("Last month", (mc.lastMonth / max).toFloat(), KhataColors.rule)
                }
            }
            HorizontalDivider(color = KhataColors.hairline)
            Row(Modifier.fillMaxWidth()) {
                Column(Modifier.weight(1f)) {
                    Text("Average a day", style = MaterialTheme.typography.bodySmall, color = KhataColors.text2)
                    KhataAmount(analysis.avgDailySpend, size = 15.sp)
                }
                Column(Modifier.weight(1f)) {
                    Text("Transactions", style = MaterialTheme.typography.bodySmall, color = KhataColors.text2)
                    Text("${analysis.totalTransactions}", style = FigureStyle.copy(fontSize = 15.sp), color = KhataColors.text)
                }
            }
        }
    }
}

@Composable
private fun CompareBar(label: String, fraction: Float, color: androidx.compose.ui.graphics.Color) {
    Row(verticalAlignment = Alignment.CenterVertically) {
        Text(label, style = MaterialTheme.typography.bodySmall, color = KhataColors.text2, modifier = Modifier.width(84.dp))
        Box(Modifier.weight(1f).height(8.dp).clip(KhataRadius.pill).background(KhataColors.surface3)) {
            Box(Modifier.fillMaxWidth(fraction.coerceIn(0f, 1f)).fillMaxHeight().clip(KhataRadius.pill).background(color))
        }
    }
}
