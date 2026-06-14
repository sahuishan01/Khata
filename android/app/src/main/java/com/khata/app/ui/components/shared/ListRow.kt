package com.khata.app.ui.components.shared

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.khata.app.ui.theme.KhataColors

@Composable
fun KhataListRow(
    leading: (@Composable () -> Unit)? = null,
    content: @Composable ColumnScope.() -> Unit,
    trailing: (@Composable () -> Unit)? = null,
    onClick: (() -> Unit)? = null,
    modifier: Modifier = Modifier,
) {
    Column(modifier = modifier) {
        Row(
            modifier = Modifier.fillMaxWidth().then(if (onClick != null) Modifier.clickable { onClick() } else Modifier)
                .padding(horizontal = 16.dp, vertical = 12.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            if (leading != null) { leading(); Spacer(Modifier.width(12.dp)) }
            Column(modifier = Modifier.weight(1f), content = content)
            if (trailing != null) { Spacer(Modifier.width(12.dp)); trailing() }
        }
        HorizontalDivider(color = KhataColors.hairline, thickness = 1.dp)
    }
}

@Composable
fun KhataListRowText(primary: String, secondary: String? = null) {
    Text(primary, fontSize = 13.5.sp, fontWeight = FontWeight.SemiBold, color = KhataColors.text, maxLines = 2, overflow = TextOverflow.Ellipsis)
    if (secondary != null) {
        Text(secondary, fontSize = 12.sp, color = KhataColors.text2, modifier = Modifier.padding(top = 2.dp))
    }
}
