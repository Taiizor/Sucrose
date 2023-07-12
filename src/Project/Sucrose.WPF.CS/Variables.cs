using SMMI = Sucrose.Manager.Manage.Internal;

namespace Sucrose.WPF.CS
{
    internal static class Variables
    {
        public static string Uri
        {
            get => SMMI.WebsiteManager.GetSetting("Uri", "https://www.vegalya.com");
            set => SMMI.WebsiteManager.SetSetting("Uri", value);
        }

        public static bool Hook
        {
            get => SMMI.WebsiteManager.GetSetting("Hook", true);
            set => SMMI.WebsiteManager.SetSetting("Hook", value);
        }

        public static bool State
        {
            get => SMMI.WebsiteManager.GetSetting("State", true);
            set => SMMI.WebsiteManager.SetSetting("State", value);
        }
    }
}