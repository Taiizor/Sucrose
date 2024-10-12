﻿using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SMMRP = Sucrose.Memory.Manage.Readonly.Path;
using SMMCE = Sucrose.Memory.Manage.Constant.Engine;
using SEDEST = Skylark.Enum.DuplicateScreenType;
using SEDYST = Skylark.Enum.DisplayScreenType;
using SEEST = Skylark.Enum.ExpandScreenType;
using SEIT = Skylark.Enum.InputType;
using SESET = Skylark.Enum.StorageType;
using SESNT = Skylark.Enum.ScreenType;
using SHC = Skylark.Helper.Culture;
using SHS = Skylark.Helper.Skymath;

namespace Sucrose.Manager.Manage
{
    public static class Engine
    {
        public static Dictionary<string, string> CefArguments => SMMI.EngineSettingManager.GetSetting(SMMCE.CefArguments, new Dictionary<string, string>());

        public static int VolumeSensitivity => SHS.Clamp(SMMI.EngineSettingManager.GetSettingStable(SMMCE.VolumeSensitivity, 5), 1, 10);

        public static SEDEST DuplicateScreenType => SMMI.EngineSettingManager.GetSetting(SMMCE.DuplicateScreenType, SEDEST.Default);

        public static SEDYST DisplayScreenType => SMMI.EngineSettingManager.GetSetting(SMMCE.DisplayScreenType, SEDYST.PerDisplay);

        public static int DeveloperPort => SHS.Clamp(SMMI.EngineSettingManager.GetSettingStable(SMMCE.DeveloperPort, 0), 0, 65535);

        public static List<string> WebArguments => SMMI.EngineSettingManager.GetSetting(SMMCE.WebArguments, new List<string>());

        public static int ScreenIndex => SHS.Clamp(SMMI.EngineSettingManager.GetSettingStable(SMMCE.ScreenIndex, 0), 0, 100);

        public static SEEST ExpandScreenType => SMMI.EngineSettingManager.GetSetting(SMMCE.ExpandScreenType, SEEST.Default);

        public static int Volume => SHS.Clamp(SMMI.EngineSettingManager.GetSettingStable(SMMCE.Volume, 100), 0, 100);

        public static SESNT ScreenType => SMMI.EngineSettingManager.GetSetting(SMMCE.ScreenType, SESNT.DisplayBound);

        public static SEIT InputType => SMMI.EngineSettingManager.GetSetting(SMMCE.InputType, SEIT.MouseKeyboard);

        public static bool VolumeDesktop => SMMI.EngineSettingManager.GetSetting(SMMCE.VolumeDesktop, false);

        public static bool DeveloperMode => SMMI.EngineSettingManager.GetSetting(SMMCE.DeveloperMode, false);

        public static bool VolumeActive => SMMI.EngineSettingManager.GetSetting(SMMCE.VolumeActive, false);

        public static bool InputDesktop => SMMI.EngineSettingManager.GetSetting(SMMCE.InputDesktop, false);

        public static bool LibraryStart => SMMI.EngineSettingManager.GetSetting(SMMCE.LibraryStart, true);

        public static bool StoreStart => SMMI.EngineSettingManager.GetSetting(SMMCE.StoreStart, true);

        public static bool Shuffle => SMMI.EngineSettingManager.GetSetting(SMMCE.Shuffle, true);

        public static bool Loop => SMMI.EngineSettingManager.GetSetting(SMMCE.Loop, true);
    }
}