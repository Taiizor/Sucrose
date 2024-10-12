using SMMCE = Sucrose.Memory.Manage.Constant.Engine;
using SMMI = Sucrose.Manager.Manage.Internal;
using SSDEET = Sucrose.Shared.Dependency.Enum.EngineType;
using SSDEIMT = Sucrose.Shared.Dependency.Enum.InputModuleType;
using SSDEST = Sucrose.Shared.Dependency.Enum.StretchType;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;

namespace Sucrose.Shared.Dependency.Manage.Manager
{
    internal static class Engine
    {
        public static SSDEET ApplicationEngine => SMMI.EngineSettingManager.GetSetting(SMMCE.ApplicationEngine, (SSDEET)SSSMI.ApplicationEngine);

        public static SSDEET YouTubeEngine => SMMI.EngineSettingManager.GetSetting(SMMCE.YouTubeEngine, (SSDEET)SSSMI.YouTubeEngine);

        public static SSDEET VideoEngine => SMMI.EngineSettingManager.GetSetting(SMMCE.VideoEngine, (SSDEET)SSSMI.VideoEngine);

        public static SSDEIMT InputModuleType => SMMI.EngineSettingManager.GetSetting(SMMCE.InputModuleType, SSDEIMT.RawInput);

        public static SSDEET WebEngine => SMMI.EngineSettingManager.GetSetting(SMMCE.WebEngine, (SSDEET)SSSMI.WebEngine);

        public static SSDEET UrlEngine => SMMI.EngineSettingManager.GetSetting(SMMCE.UrlEngine, (SSDEET)SSSMI.UrlEngine);

        public static SSDEET GifEngine => SMMI.EngineSettingManager.GetSetting(SMMCE.GifEngine, (SSDEET)SSSMI.GifEngine);

        public static SSDEST StretchType => SMMI.EngineSettingManager.GetSetting(SMMCE.StretchType, SSDEST.Fill);
    }
}