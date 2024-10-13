using SHS = Skylark.Helper.Skymath;
using SMMCH = Sucrose.Memory.Manage.Constant.Hook;
using SMMI = Sucrose.Manager.Manage.Internal;

namespace Sucrose.Manager.Manage
{
    public static class Hook
    {
        public static int DiscordRefreshDelay => SHS.Clamp(SMMI.HookSettingManager.GetSettingStable(SMMCH.DiscordRefreshDelay, 60), 60, 3600);

        public static bool DiscordRefreshConnect => SMMI.HookSettingManager.GetSetting(SMMCH.DiscordRefreshConnect, true);

        public static bool DiscordConnect => SMMI.HookSettingManager.GetSetting(SMMCH.DiscordConnect, true);
    }
}