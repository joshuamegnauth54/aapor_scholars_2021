#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReviewType {
    All,
    Positive,
    Negative,
}

impl ReviewType {
    pub fn as_str(self) -> &'static str {
        use ReviewType::*;
        match self {
            All => "all",
            Positive => "positive",
            Negative => "negative",
        }
    }
}

impl Default for ReviewType {
    fn default() -> Self {
        ReviewType::All
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Filter {
    Recent,
    Updated,
    All,
}

impl Filter {
    pub fn as_str(self) -> &'static str {
        use Filter::*;
        match self {
            Recent => "recent",
            Updated => "updated",
            All => "all",
        }
    }
}

impl Default for Filter {
    fn default() -> Self {
        Filter::All
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PurchaseType {
    All,
    NonSteamPurchase,
    Steam,
}

impl PurchaseType {
    pub fn as_str(self) -> &'static str {
        use PurchaseType::*;
        match self {
            All => "all",
            NonSteamPurchase => "non_steam_purchase",
            Steam => "steam",
        }
    }
}

impl Default for PurchaseType {
    fn default() -> Self {
        PurchaseType::Steam
    }
}
