using SHS = Skylark.Helper.Skymath;
using SMMCH = Sucrose.Memory.Manage.Constant.Hook;
using SMMI = Sucrose.Manager.Manage.Internal;

namespace Sucrose.Manager.Manage
{
    public static class Hook
    {
        public static int DiscordDelay => SHS.Clamp(SMMI.HookSettingManager.GetSettingStable(SMMCH.DiscordDelay, 60), 60, 3600);

        public static bool DiscordRefresh => SMMI.HookSettingManager.GetSetting(SMMCH.DiscordRefresh, true);

        public static bool DiscordState => SMMI.HookSettingManager.GetSetting(SMMCH.DiscordState, true);
    }
}