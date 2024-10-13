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
        public static SSDEET Application => SMMI.EngineSettingManager.GetSetting(SMMCE.Application, (SSDEET)SSSMI.ApplicationEngine);

        public static SSDEIMT InputModuleType => SMMI.EngineSettingManager.GetSetting(SMMCE.InputModuleType, SSDEIMT.RawInput);

        public static SSDEET YouTube => SMMI.EngineSettingManager.GetSetting(SMMCE.YouTube, (SSDEET)SSSMI.YouTubeEngine);

        public static SSDEET Video => SMMI.EngineSettingManager.GetSetting(SMMCE.Video, (SSDEET)SSSMI.VideoEngine);

        public static SSDEST StretchType => SMMI.EngineSettingManager.GetSetting(SMMCE.StretchType, SSDEST.Fill);

        public static SSDEET Web => SMMI.EngineSettingManager.GetSetting(SMMCE.Web, (SSDEET)SSSMI.WebEngine);

        public static SSDEET Url => SMMI.EngineSettingManager.GetSetting(SMMCE.Url, (SSDEET)SSSMI.UrlEngine);

        public static SSDEET Gif => SMMI.EngineSettingManager.GetSetting(SMMCE.Gif, (SSDEET)SSSMI.GifEngine);
    }
}