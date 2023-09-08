using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SSDEET = Sucrose.Shared.Dependency.Enum.EngineType;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;

namespace Sucrose.Shared.Live.Manage
{
    internal static class Manager
    {
        public static SSDEET AApp => SMMI.EngineSettingManager.GetSetting(SMC.AApp, (SSDEET)SSSMI.ApplicationEngine);

        public static SSDEET YApp => SMMI.EngineSettingManager.GetSetting(SMC.YApp, (SSDEET)SSSMI.YouTubeEngine);

        public static SSDEET VApp => SMMI.EngineSettingManager.GetSetting(SMC.VApp, (SSDEET)SSSMI.VideoEngine);

        public static SSDEET GApp => SMMI.EngineSettingManager.GetSetting(SMC.GApp, (SSDEET)SSSMI.GifEngine);

        public static SSDEET UApp => SMMI.EngineSettingManager.GetSetting(SMC.UApp, (SSDEET)SSSMI.UrlEngine);

        public static SSDEET WApp => SMMI.EngineSettingManager.GetSetting(SMC.WApp, (SSDEET)SSSMI.WebEngine);
    }
}