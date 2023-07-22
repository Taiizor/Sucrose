using Newtonsoft.Json.Linq;
using SEDST = Skylark.Enum.DuplicateScreenType;
using SEEST = Skylark.Enum.ExpandScreenType;
using SEST = Skylark.Enum.ScreenType;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SSDEDT = Sucrose.Shared.Dependency.Enum.DisplayType;
using SSDEST = Sucrose.Shared.Dependency.Enum.StretchType;

namespace Sucrose.Shared.Engine.Helper
{
    internal static class Data
    {
        public static bool GetLoop()
        {
            return SMMI.EngineSettingManager.GetSetting(SMC.Loop, true);
        }

        public static int GetVolume()
        {
            return SMMI.EngineSettingManager.GetSettingStable(SMC.Volume, 100);
        }

        public static bool GetShuffle()
        {
            return SMMI.EngineSettingManager.GetSetting(SMC.Shuffle, true);
        }

        public static SSDEST GetStretch()
        {
            SSDEST Stretch = SMMI.EngineSettingManager.GetSetting(SMC.StretchType, SSDEST.Fill);

            if ((int)Stretch < Enum.GetValues(typeof(SSDEST)).Length)
            {
                return Stretch;
            }
            else
            {
                return SSDEST.None;
            }
        }

        public static SEST GetScreenType()
        {
            return SMMI.EngineSettingManager.GetSetting(SMC.ScreenType, SEST.DisplayBound);
        }

        public static int GetScreenIndex()
        {
            return SMMI.EngineSettingManager.GetSettingStable(SMC.ScreenIndex, 0);
        }

        public static SSDEDT GetDisplayType()
        {
            return SMMI.EngineSettingManager.GetSetting(SMC.DisplayType, SSDEDT.Screen);
        }

        public static JObject GetComputerDate()
        {
            DateTime Date = DateTime.Now;

            return new
            (
                new JProperty("Year", Date.Year),
                new JProperty("Month", Date.Month),
                new JProperty("Day", Date.Day),
                new JProperty("Hour", Date.Hour),
                new JProperty("Minute", Date.Minute),
                new JProperty("Second", Date.Second)
            );
        }

        public static SEEST GetExpandScreenType()
        {
            return SMMI.EngineSettingManager.GetSetting(SMC.ExpandScreenType, SEEST.Default);
        }

        public static SEDST GetDuplicateScreenType()
        {
            return SMMI.EngineSettingManager.GetSetting(SMC.DuplicateScreenType, SEDST.Default);
        }
    }
}