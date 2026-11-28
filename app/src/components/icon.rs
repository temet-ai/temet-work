//! Icon (`blueprint-icon`): one of Blueprint's icons, drawn from the SVG files embedded in the
//! app (`app/assets/icons`, served by `crate::assets`).

use gpui::{App, Hsla, IntoElement, Pixels, RenderOnce, SharedString, Styled, Window, svg};

use crate::theme;

/// The Blueprint icons the app's parts use, by Blueprint's names for them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IconName {
    // StatusTag's statuses.
    TickCircle,
    Refresh,
    Time,
    Pause,
    Issue,
    BanCircle,
    CrossCircle,
    // StageTag's stages.
    Inbox,
    Edit,
    Clipboard,
    Code,
    EyeOpen,
    Wrench,
    TakeAction,
    GitMerge,
    Rocket,
    LabTest,
    Tick,
    // PlusMinusTree's groups.
    Plus,
    Minus,
    // PageHeader's facts.
    Link,
    GitBranch,
    // NeedsYou's title, a decision's cost to undo and the Overturn toast.
    Notifications,
    WarningSign,
    InfoSign,
    // A toast's close button, from Blueprint's newer icon set.
    X,
    // Projects in SideNav.
    Console,
    Database,
    Layout,
    Desktop,
    Box,
    Search,
    // A project in SideNav, and closing a pane.
    GitRepo,
    Cross,
}

impl IconName {
    /// Every icon the app embeds.
    pub const ALL: [IconName; 34] = [
        Self::TickCircle,
        Self::Refresh,
        Self::Time,
        Self::Pause,
        Self::Issue,
        Self::BanCircle,
        Self::CrossCircle,
        Self::Inbox,
        Self::Edit,
        Self::Clipboard,
        Self::Code,
        Self::EyeOpen,
        Self::Wrench,
        Self::TakeAction,
        Self::GitMerge,
        Self::Rocket,
        Self::LabTest,
        Self::Tick,
        Self::Plus,
        Self::Minus,
        Self::Link,
        Self::GitBranch,
        Self::Notifications,
        Self::WarningSign,
        Self::InfoSign,
        Self::X,
        Self::Console,
        Self::Database,
        Self::Layout,
        Self::Desktop,
        Self::Box,
        Self::Search,
        Self::GitRepo,
        Self::Cross,
    ];

    /// Blueprint's name for the icon, such as `tick-circle`.
    pub fn name(self) -> &'static str {
        match self {
            Self::TickCircle => "tick-circle",
            Self::Refresh => "refresh",
            Self::Time => "time",
            Self::Pause => "pause",
            Self::Issue => "issue",
            Self::BanCircle => "ban-circle",
            Self::CrossCircle => "cross-circle",
            Self::Inbox => "inbox",
            Self::Edit => "edit",
            Self::Clipboard => "clipboard",
            Self::Code => "code",
            Self::EyeOpen => "eye-open",
            Self::Wrench => "wrench",
            Self::TakeAction => "take-action",
            Self::GitMerge => "git-merge",
            Self::Rocket => "rocket",
            Self::LabTest => "lab-test",
            Self::Tick => "tick",
            Self::Plus => "plus",
            Self::Minus => "minus",
            Self::Link => "link",
            Self::GitBranch => "git-branch",
            Self::Notifications => "notifications",
            Self::WarningSign => "warning-sign",
            Self::InfoSign => "info-sign",
            Self::X => "x",
            Self::Console => "console",
            Self::Database => "database",
            Self::Layout => "layout",
            Self::Desktop => "desktop",
            Self::Box => "box",
            Self::Search => "search",
            Self::GitRepo => "git-repo",
            Self::Cross => "cross",
        }
    }

    /// The icon with Blueprint's name `name`, if the app embeds it.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|i| i.name() == name)
    }

    /// The asset path of the icon's SVG on Blueprint's 16 px or 20 px grid.
    pub fn path(self, size: IconSize) -> SharedString {
        let grid = match size {
            IconSize::Standard => "16",
            IconSize::Large => "20",
        };
        format!("icons/{grid}/{}.svg", self.name()).into()
    }
}

/// Blueprint's icon sizes: `IconSize.STANDARD` and `IconSize.LARGE`, each drawn from the
/// paths for its own grid.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IconSize {
    #[default]
    Standard,
    Large,
}

/// An icon. Without a colour it takes the colour of the text around it, as Blueprint's icons
/// take `currentColor`.
#[derive(IntoElement)]
pub struct Icon {
    icon: IconName,
    size: IconSize,
    side: Option<Pixels>,
    color: Option<Hsla>,
}

impl Icon {
    pub fn new(icon: IconName) -> Self {
        Self {
            icon,
            size: IconSize::Standard,
            side: None,
            color: None,
        }
    }

    /// The size, on Blueprint's 16 px or 20 px grid.
    pub fn size(mut self, size: IconSize) -> Self {
        self.size = size;
        self
    }

    /// Draws a large icon at another side, as a part's own rules size its icon with
    /// `font-size`, such as the Alert's, twice the large size.
    pub(crate) fn large_at(mut self, side: Pixels) -> Self {
        self.size = IconSize::Large;
        self.side = Some(side);
        self
    }

    /// The icon's colour, in place of the text colour around it.
    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }
}

impl RenderOnce for Icon {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let side = self.side.unwrap_or_else(|| match self.size {
            IconSize::Standard => theme::size::icon(cx),
            IconSize::Large => theme::size::icon_large(cx),
        });
        let color = self.color.unwrap_or_else(|| window.text_style().color);
        svg()
            .path(self.icon.path(self.size))
            .size(side)
            .flex_none()
            .text_color(color)
    }
}
