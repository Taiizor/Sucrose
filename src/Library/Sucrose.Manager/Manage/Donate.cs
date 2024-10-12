using SEDEST = Skylark.Enum.DuplicateScreenType;
using SEDYST = Skylark.Enum.DisplayScreenType;
using SEEST = Skylark.Enum.ExpandScreenType;
using SEIT = Skylark.Enum.InputType;
using SESET = Skylark.Enum.StorageType;
using SESNT = Skylark.Enum.ScreenType;
using SHC = Skylark.Helper.Culture;
using SHS = Skylark.Helper.Skymath;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SMMRP = Sucrose.Memory.Manage.Readonly.Path;
using SMMCH = Sucrose.Memory.Manage.Constant.Hook;
using SMMCU = Sucrose.Memory.Manage.Constant.User;
using SMMCA = Sucrose.Memory.Manage.Constant.Aurora;
using SMMCD = Sucrose.Memory.Manage.Constant.Donate;

namespace Sucrose.Manager.Manage
{
    public static class Donate
    {
        public static int AdvertisingDelay => SHS.Clamp(SMMI.DonateSettingManager.GetSettingStable(SMMCD.AdvertisingDelay, 30), 30, 720);

        public static bool AdvertisingState => SMMI.DonateSettingManager.GetSetting(SMMCD.AdvertisingState, true);

        public static bool DonateVisible => SMMI.DonateSettingManager.GetSetting(SMMCD.DonateVisible, true);
    }
}