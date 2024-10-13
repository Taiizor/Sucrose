using SHS = Skylark.Helper.Skymath;
using SMMCP = Sucrose.Memory.Manage.Constant.Portal;
using SMMI = Sucrose.Manager.Manage.Internal;

namespace Sucrose.Manager.Manage
{
    public static class Portal
    {
        public static int BackgroundOpacity => SHS.Clamp(SMMI.PortalSettingManager.GetSettingStable(SMMCP.BackgroundOpacity, 100), 0, 100);

        public static int LibraryPagination => SHS.Clamp(SMMI.PortalSettingManager.GetSettingStable(SMMCP.LibraryPagination, 30), 1, 100);

        public static int StorePagination => SHS.Clamp(SMMI.PortalSettingManager.GetSettingStable(SMMCP.StorePagination, 30), 1, 100);

        public static int AdaptiveLayout => SHS.Clamp(SMMI.PortalSettingManager.GetSettingStable(SMMCP.AdaptiveLayout, 0), 0, 100);

        public static int AdaptiveMargin => SHS.Clamp(SMMI.PortalSettingManager.GetSettingStable(SMMCP.AdaptiveMargin, 5), 5, 25);

        public static int StoreDuration => SHS.Clamp(SMMI.PortalSettingManager.GetSettingStable(SMMCP.StoreDuration, 3), 1, 24);

        public static string BackgroundImage => SMMI.PortalSettingManager.GetSetting(SMMCP.BackgroundImage, string.Empty);

        public static bool LibraryPreviewHide => SMMI.PortalSettingManager.GetSetting(SMMCP.LibraryPreviewHide, false);

        public static bool StorePreviewHide => SMMI.PortalSettingManager.GetSetting(SMMCP.StorePreviewHide, false);

        public static bool LibraryPreview => SMMI.PortalSettingManager.GetSetting(SMMCP.LibraryPreview, false);

        public static bool StorePreview => SMMI.PortalSettingManager.GetSetting(SMMCP.StorePreview, false);

        public static bool StoreAdult => SMMI.PortalSettingManager.GetSetting(SMMCP.StoreAdult, true);
    }
}