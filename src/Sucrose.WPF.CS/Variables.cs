using Sucrose.Common.Manage;

namespace Sucrose.WPF.CS
{
    internal static class Variables
    {
        public static string Uri
        {
            get => Internal.WebsiteManager.GetSetting<string>("Uri") ?? "https://www.vegalya.com";
            set => Internal.WebsiteManager.SetSetting("Uri", value);
        }

        public static bool Hook
        {
            get => Internal.WebsiteManager.GetSetting<bool>("Hook");
            set => Internal.WebsiteManager.SetSetting("Hook", value);
        }

        public static bool State
        {
            get => Internal.WebsiteManager.GetSetting<bool>("State");
            set => Internal.WebsiteManager.SetSetting("State", value);
        }
    }
}
