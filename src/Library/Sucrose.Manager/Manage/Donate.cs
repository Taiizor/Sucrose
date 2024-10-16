using SHS = Skylark.Helper.Skymath;
using SMMCD = Sucrose.Memory.Manage.Constant.Donate;
using SMMI = Sucrose.Manager.Manage.Internal;

namespace Sucrose.Manager.Manage
{
    public static class Donate
    {
        public static int AdvertisingDelay => SHS.Clamp(SMMI.DonateSettingManager.GetSettingStable(SMMCD.AdvertisingDelay, 30), 30, 720);

        public static bool AdvertisingActive => SMMI.DonateSettingManager.GetSetting(SMMCD.AdvertisingActive, true);

        public static bool MenuVisible => SMMI.DonateSettingManager.GetSetting(SMMCD.MenuVisible, true);
    }
}