using SMMCO = Sucrose.Memory.Manage.Constant.Objectionable;
using SMMI = Sucrose.Manager.Manage.Internal;

namespace Sucrose.Manager.Manage
{
    public static class Objectionable
    {
        public static string Key => SMMI.ObjectionableSettingManager.GetSetting(SMMCO.Key, string.Empty);
    }
}