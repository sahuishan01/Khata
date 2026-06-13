package com.khata.app.util

fun maskIdentifier(id: String): String {
    if (id.length <= 4) return "••••"
    return "••••${id.takeLast(4)}"
}

fun maskDescription(desc: String, blur: Boolean): String {
    if (!blur) return desc
    if (desc.length <= 8) return "••••••••"
    return desc.take(4) + "••••" + desc.takeLast(4)
}

fun maskEmail(email: String, blur: Boolean): String {
    if (!blur) return email
    val parts = email.split("@", limit = 2)
    if (parts.size != 2) return email
    return "${parts[0].take(2)}•••@${parts[1]}"
}
