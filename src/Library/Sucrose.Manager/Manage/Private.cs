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
using SMMCO = Sucrose.Memory.Manage.Constant.Objectionable;

namespace Sucrose.Manager.Manage
{
    public static class Objectionable
    {
        public static string Key => SMMI.ObjectionableSettingManager.GetSetting(SMMCO.Key, SMR.Key);
    }
}