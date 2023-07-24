using System.IO;
using SaveFileDialog = Microsoft.Win32.SaveFileDialog;
using SGHLL = Sucrose.Globalization.Helper.LauncherLocalization;
using SMR = Sucrose.Memory.Readonly;
using SMV = Sucrose.Memory.Valuable;
using SSZEZ = Sucrose.Shared.Zip.Extension.Zip;

namespace Sucrose.Shared.Launcher.Helper
{
    internal static class Create
    {
        public static void Start()
        {
            SaveFileDialog SaveDialog = new()
            {
                FileName = SMV.LogCompress,

                Filter = SGHLL.GetValue("SaveDialogFilter"),
                FilterIndex = 1,

                Title = SGHLL.GetValue("SaveDialogTitle"),

                InitialDirectory = SMR.DesktopPath
            };

            if (SaveDialog.ShowDialog() == true)
            {
                string Destination = SaveDialog.FileName;

                string[] Sources = new[]
                {
                    Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.LogFolder),
                    Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.SettingFolder)
                };

                SSZEZ.Compress(Sources, Destination);
            }
        }
    }
}