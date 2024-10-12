﻿using SMLM = Sucrose.Manager.LogManager;
using SMSM = Sucrose.Manager.SettingManager;

namespace Sucrose.Manager.Manage
{
    public static class Internal
    {
        public static readonly SMSM CoreSettingManager = new("Core.json");

        public static readonly SMSM HookSettingManager = new("Hook.json");

        public static readonly SMSM UserSettingManager = new("User.json");

        public static readonly SMLM PortalLogManager = new("Portal-{0}.log");

        public static readonly SMLM UpdateLogManager = new("Update-{0}.log");

        public static readonly SMSM AuroraSettingManager = new("Aurora.json");

        public static readonly SMSM DonateSettingManager = new("Donate.json");

        public static readonly SMSM EngineSettingManager = new("Engine.json");

        public static readonly SMSM PortalSettingManager = new("Portal.json");

        public static readonly SMSM SystemSettingManager = new("System.json");

        public static readonly SMSM UpdateSettingManager = new("Update.json");

        public static readonly SMSM CyclingSettingManager = new("Cycling.json");

        public static readonly SMSM GeneralSettingManager = new("General.json");

        public static readonly SMSM LibrarySettingManager = new("Library.json");

        public static readonly SMLM LauncherLogManager = new("Launcher-{0}.log");

        public static readonly SMLM PropertyLogManager = new("Property-{0}.log");

        public static readonly SMLM WatchdogLogManager = new("Watchdog-{0}.log");

        public static readonly SMSM LauncherSettingManager = new("Launcher.json");

        public static readonly SMLM CommandogLogManager = new("Commandog-{0}.log");

        public static readonly SMLM ReportdogLogManager = new("Reportdog-{0}.log");

        public static readonly SMLM AuroraLiveLogManager = new("AuroraLive-{0}.log");

        public static readonly SMLM NebulaLiveLogManager = new("NebulaLive-{0}.log");

        public static readonly SMLM VexanaLiveLogManager = new("VexanaLive-{0}.log");

        public static readonly SMLM XavierLiveLogManager = new("XavierLive-{0}.log");

        public static readonly SMLM WebViewLiveLogManager = new("WebViewLive-{0}.log");

        public static readonly SMLM BackgroundogLogManager = new("Backgroundog-{0}.log");

        public static readonly SMLM CefSharpLiveLogManager = new("CefSharpLive-{0}.log");

        public static readonly SMSM BackgroundogSettingManager = new("Backgroundog.json");

        public static readonly SMLM MpvPlayerLiveLogManager = new("MpvPlayerLive-{0}.log");

        public static readonly SMSM ObjectionableSettingManager = new("Objectionable.json");
    }
}