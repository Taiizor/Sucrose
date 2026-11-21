using System.IO;
using SMMRF = Sucrose.Memory.Manage.Readonly.Folder;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;
using SMMRP = Sucrose.Memory.Manage.Readonly.Path;
using SRER = Sucrose.Resources.Extension.Resources;
using SWML = Skylark.Wing.Manage.LockScreen;

namespace Sucrose.Undo.Manage
{
    internal static class Internal
    {
        public static string Message => SRER.GetValue("Undo", "QuestionMessage") + Environment.NewLine + Environment.NewLine + SRER.GetValue("Undo", "QuestionDescription");

        public static string Runtime => Path.Combine(UninstallPath, $"{SMMRG.AppName}.{SMMRF.Runtime}");

        public static string UninstallPath => Path.Combine(SMMRP.LocalApplicationData, SMMRG.AppName);

        public static string UninstallDataPath => Path.Combine(SMMRP.ApplicationData, SMMRG.AppName);

        public static string LockScreenDataPath => Path.Combine(SMMRP.CommonApplicationData, SMMRG.AppName, SMMRF.LockScreen);

        public static string RegistryName => @"Software\Microsoft\Windows\CurrentVersion\Uninstall";

        public static string BatchFile = Path.Combine(SMMRP.Temp, $"del_{Guid.NewGuid():N}.bat");

        public static string StartMenu => Path.Combine(SMMRP.StartMenu, "Programs", Shortcut);

        public static string Undo => Path.Combine(UninstallPath, $"{SMMRG.AppName}.Undo");

        public static string Desktop => Path.Combine(SMMRP.Desktop, Shortcut);

        public static string Title => SRER.GetValue("Undo", "QuestionTitle");

        public static string Shortcut => $"{SMMRG.AppLongName}.lnk";

        public static int Delay => 1000;
    }
}