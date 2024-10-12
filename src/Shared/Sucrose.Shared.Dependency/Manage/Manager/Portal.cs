using System.Windows.Media;
using SMMCP = Sucrose.Memory.Manage.Constant.Portal;
using SMMI = Sucrose.Manager.Manage.Internal;
using SSDESKT = Sucrose.Shared.Dependency.Enum.SortKindType;
using SSDESMT = Sucrose.Shared.Dependency.Enum.SortModeType;
using SSDESST = Sucrose.Shared.Dependency.Enum.StoreServerType;
using SSDMI = Sucrose.Shared.Dependency.Manage.Internal;

namespace Sucrose.Shared.Dependency.Manage.Manager
{
    internal static class Portal
    {
        public static Stretch BackgroundStretch => SMMI.PortalSettingManager.GetSetting(SMMCP.BackgroundStretch, SSDMI.DefaultBackgroundStretch);

        public static SSDESKT LibrarySortKind => SMMI.PortalSettingManager.GetSetting(SMMCP.LibrarySortKind, SSDESKT.Descending);

        public static SSDESST StoreServerType => SMMI.PortalSettingManager.GetSetting(SMMCP.StoreServerType, SSDESST.Soferity);

        public static SSDESMT LibrarySortMode => SMMI.PortalSettingManager.GetSetting(SMMCP.LibrarySortMode, SSDESMT.Creation);
    }
}