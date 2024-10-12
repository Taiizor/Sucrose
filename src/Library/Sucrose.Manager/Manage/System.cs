using SMMCS = Sucrose.Memory.Manage.Constant.System;
using SMMI = Sucrose.Manager.Manage.Internal;

namespace Sucrose.Manager.Manage
{
    public static class System
    {
        public static string[] NetworkInterfaces => SMMI.SystemSettingManager.GetSetting(SMMCS.NetworkInterfaces, Array.Empty<string>());

        public static string[] GraphicInterfaces => SMMI.SystemSettingManager.GetSetting(SMMCS.GraphicInterfaces, Array.Empty<string>());
    }
}