using SHS = Skylark.Helper.Skymath;
using SMMCD = Sucrose.Memory.Manage.Constant.Donate;
using SMMI = Sucrose.Manager.Manage.Internal;

namespace Sucrose.Manager.Manage
{
    public static class Donate
    {
        public static int AdvertisingDelay => SHS.Clamp(SMMI.DonateSettingManager.GetSettingStable(SMMCD.AdvertisingDelay, 30), 30, 720);

        public static bool AdvertisingState => SMMI.DonateSettingManager.GetSetting(SMMCD.AdvertisingState, true);

        public static bool DonateVisible => SMMI.DonateSettingManager.GetSetting(SMMCD.DonateVisible, true);
    }
}