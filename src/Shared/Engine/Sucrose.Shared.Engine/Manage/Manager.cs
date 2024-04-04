using SEDEST = Skylark.Enum.DuplicateScreenType;
using SEDYST = Skylark.Enum.DisplayScreenType;
using SEEST = Skylark.Enum.ExpandScreenType;
using SEST = Skylark.Enum.ScreenType;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommunicationType;
using SSDEST = Sucrose.Shared.Dependency.Enum.StretchType;

namespace Sucrose.Shared.Engine.Manage
{
    internal static class Manager
    {
        public static SSDECT CommunicationType => SMMI.BackgroundogSettingManager.GetSetting(SMC.CommunicationType, SSDECT.Signal);

        public static SEDEST DuplicateScreenType => SMMI.EngineSettingManager.GetSetting(SMC.DuplicateScreenType, SEDEST.Default);

        public static SEDYST DisplayScreenType => SMMI.EngineSettingManager.GetSetting(SMC.DisplayScreenType, SEDYST.PerDisplay);

        public static SEEST ExpandScreenType => SMMI.EngineSettingManager.GetSetting(SMC.ExpandScreenType, SEEST.Default);

        public static SEST ScreenType => SMMI.EngineSettingManager.GetSetting(SMC.ScreenType, SEST.DisplayBound);

        public static SSDEST StretchType => SMMI.EngineSettingManager.GetSetting(SMC.StretchType, SSDEST.Fill);

        public static SSDEST DefaultStretchType => SSDEST.None;
    }
}