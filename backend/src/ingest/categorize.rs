/// Keyword-based transaction categorizer.
/// Rules are checked in order — first match wins. Miscellaneous is the fallback.
pub fn categorize(description: &str, direction: &str) -> &'static str {
    let d = description.to_uppercase();

    // Credit-specific overrides (salary, refund, interest)
    if direction == "credit" {
        if contains_any(&d, &["SALARY", "PAYROLL", "STIPEND"]) {
            return "Salary";
        }
        if contains_any(&d, &["REFUND", "CASHBACK", "REVERSAL"]) {
            return "Refund";
        }
        if contains_any(&d, &["INTEREST", "INT CREDIT", "FD INT"]) {
            return "Interest";
        }
        if contains_any(&d, &["DIVIDEND"]) {
            return "Investment";
        }
    }

    // Debit-specific overrides
    if direction == "debit" {
        if contains_any(&d, &["ATM", "CASH WITHDRAWAL", "CASH WDL", "CASH W/D"]) {
            return "ATM/Cash";
        }
        if contains_any(&d, &["EMI", "LOAN REPAY", "EQUATED MONTHLY", "HOME LOAN", "CAR LOAN", "PERSONAL LOAN"]) {
            return "EMI/Loan";
        }
    }

    // Direction-neutral rules (ordered by specificity)
    let rules: &[(&[&str], &str)] = &[
        // Food & Dining
        (&["SWIGGY", "ZOMATO", "DUNZO", "BLINKIT FOOD", "RESTAURANT", "CAFE", "CANTEEN",
           "DOMINOS", "DOMINO", "MCDONALD", "KFC", "PIZZA HUT", "SUBWAY", "BURGER KING",
           "STARBUCKS", "CHAI POINT", "BIRYANI", "DHABA", "HOTEL FOOD", "HALDIRAM",
           "BARBEQUE", "BBQ"], "Food & Dining"),

        // Groceries
        (&["BIGBASKET", "BIG BASKET", "BLINKIT", "ZEPTO", "GROFER", "JIOMART",
           "RELIANCE FRESH", "RELIANCE SMART", "DMART", "D-MART", "MORE SUPERMARKET",
           "SPAR", "NATURE BASKET", "VEGETABLES", "FRUITS", "GROCERY", "KIRANA",
           "SUPERMARKET", "HYPERMARKET", "APNIDUKAAN"], "Groceries"),

        // Shopping
        (&["AMAZON", "FLIPKART", "MYNTRA", "AJIO", "MEESHO", "NYKAA", "SNAPDEAL",
           "TATA CLiQ", "TATACLIQ", "SHOPSY", "DECATHLON", "LIFESTYLE", "WESTSIDE",
           "ZARA", "H&M", "PANTALOONS", "SHOPPERS STOP", "RELIANCE TRENDS",
           "MALL", "ONLINE SHOPPING"], "Shopping"),

        // Travel
        (&["IRCTC", "INDIAN RAILWAY", "RAILWAYS", "REDBUS", "MAKEMYTRIP", "GOIBIBO",
           "CLEARTRIP", "YATRA", "EASE MY TRIP", "INDIGO", "SPICEJET", "AIR INDIA",
           "VISTARA", "AIRINDIA", "IXIGO", "FLIGHT", "TRAIN TICKET",
           "OYO", "TREEBO", "FABHOTELS", "HOTEL BOOKING"], "Travel"),

        // Transport
        (&["OLA", "UBER", "RAPIDO", "NAMMA YATRI", "BLUMART", "METRO RAIL",
           "MUMBAI METRO", "DMRC", "BMTC", "BEST BUS", "AUTO RICKSHAW",
           "RICKSHAW", "TAXI", "CAB "], "Transport"),

        // Fuel
        (&["PETROL", "DIESEL", "HPCL", "BPCL", "IOCL", "HP PETRO", "BHARAT PETRO",
           "INDIAN OIL", "ESSAR PETRO", "SHELL PETRO", "NAYARA", "FUEL STATION",
           "FILLING STATION"], "Fuel"),

        // Entertainment
        (&["NETFLIX", "HOTSTAR", "DISNEY", "AMAZON PRIME", "PRIME VIDEO", "JIOCINEMA",
           "SONYLIV", "ZEE5", "MXPLAYER", "SPOTIFY", "GAANA", "WYNK", "JIOSAAVN",
           "YOUTUBE PREMIUM", "PVR", "INOX", "CINEPOLIS", "BOOKMYSHOW",
           "LENSKART", "STEAM", "GAMING", "PLAYSTATION"], "Entertainment"),

        // Utilities & Bills
        (&["ELECTRICITY", "ELECTRIC BILL", "MSEDCL", "BESCOM", "TSSPDCL", "TNEB",
           "WATER BILL", "MAHANAGAR GAS", "IGL ", "MGL ", "PIPED GAS", "GAS BILL",
           "TATA SKY", "DISH TV", "AIRTEL DTH", "D2H ", "DTH ",
           "BROADBAND", "WIFI", "INTERNET BILL",
           "AIRTEL", "JIO ", "BSNL", "VODAFONE", "IDEA CELLULAR", "VI PREPAID",
           "MOBILE RECHARGE", "RECHARGE"], "Utilities"),

        // Health
        (&["PHARMACY", "CHEMIST", "MEDPLUS", "APOLLO PHARMACY", "NETMEDS",
           "1MG", "PHARMEASY", "TATA 1MG", "HOSPITAL", "CLINIC", "DOCTOR",
           "DIAGNOSTIC", "PATHLAB", "BLOOD TEST", "HEALTH CHECK",
           "DENTAL", "OPTICAL", "HEALTH INSURANCE"], "Health"),

        // Education
        (&["SCHOOL FEE", "COLLEGE FEE", "TUITION FEE", "COURSE FEE",
           "UDEMY", "COURSERA", "UNACADEMY", "BYJUS", "BYJU", "VEDANTU",
           "PHYSICSWALLAH", "WHITEHAT JR", "UPGRAD", "SIMPLILEARN",
           "EXAM FEE", "ADMISSION FEE"], "Education"),

        // Insurance
        (&["INSURANCE", "LIC PREMIUM", "LIC POLICY", "TERM PLAN",
           "HEALTH COVER", "MOTOR INSURANCE", "VEHICLE INSURANCE",
           "HDFC LIFE", "ICICI PRU", "BAJAJ ALLIANZ", "STAR HEALTH",
           "MAX BUPA", "TATA AIG"], "Insurance"),

        // Investment
        (&["MUTUAL FUND", "MF ", "ZERODHA", "GROWW", "KUVERA", "COIN ",
           "SIP ", "SYSTEMATIC INVESTMENT", "STOCK", "EQUITY", "IPO ",
           "PPFAS", "NIFTY", "SENSEX", "NSE ", "BSE "], "Investment"),

        // Subscriptions
        (&["SUBSCRIPTION", "ANNUAL FEE", "MEMBERSHIP FEE", "RENEWAL FEE",
           "CREDIT CARD FEE", "ANNUAL MAINTENANCE", "AMC "], "Subscriptions"),

        // Transfer (generic)
        (&["NEFT", "RTGS", "IMPS", "UPI TRANSFER", "FUND TRANSFER",
           "SELF TRANSFER", "BANK TRANSFER"], "Transfer"),
    ];

    for (keywords, category) in rules {
        if contains_any(&d, keywords) {
            return category;
        }
    }

    "Miscellaneous"
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| haystack.contains(n))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn food_delivery() {
        assert_eq!(categorize("UPI/SWIGGY TECHNOLOGIES/PAY", "debit"), "Food & Dining");
        assert_eq!(categorize("ZOMATO ORDER 12345", "debit"), "Food & Dining");
    }

    #[test]
    fn salary_credit() {
        assert_eq!(categorize("SALARY CREDIT FOR MARCH", "credit"), "Salary");
        assert_eq!(categorize("NEFT-PAYROLL-MARCH2026", "credit"), "Salary");
    }

    #[test]
    fn atm_cash() {
        assert_eq!(categorize("ATM WDL 12345 ANDHERI", "debit"), "ATM/Cash");
        assert_eq!(categorize("CASH WITHDRAWAL", "debit"), "ATM/Cash");
    }

    #[test]
    fn miscellaneous_fallback() {
        assert_eq!(categorize("SOME UNKNOWN VENDOR XYZ", "debit"), "Miscellaneous");
    }

    #[test]
    fn utilities() {
        assert_eq!(categorize("JIO RECHARGE 249", "debit"), "Utilities");
        assert_eq!(categorize("MSEDCL ELECTRICITY BILL", "debit"), "Utilities");
    }

    #[test]
    fn shopping() {
        assert_eq!(categorize("AMAZON PAY RETAIL", "debit"), "Shopping");
        assert_eq!(categorize("FLIPKART INTERNET", "debit"), "Shopping");
    }
}
