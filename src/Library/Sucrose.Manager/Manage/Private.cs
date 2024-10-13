using SMMCO = Sucrose.Memory.Manage.Constant.Objectionable;
using SMMI = Sucrose.Manager.Manage.Internal;

namespace Sucrose.Manager.Manage
{
    public static class Objectionable
    {
        public static string PersonalAccessToken => SMMI.ObjectionableSettingManager.GetSetting(SMMCO.PersonalAccessToken, string.Empty);
    }
}