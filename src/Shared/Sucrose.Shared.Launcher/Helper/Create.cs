using System.IO;
using SaveFileDialog = Microsoft.Win32.SaveFileDialog;
using SSRER = Sucrose.Shared.Resources.Extension.Resources;
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

                Filter = SSRER.GetValue("Launcher", "SaveDialogFilter"),
                FilterIndex = 1,

                Title = SSRER.GetValue("Launcher", "SaveDialogTitle"),

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