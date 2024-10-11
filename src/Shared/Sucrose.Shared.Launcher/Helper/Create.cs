using System.IO;
using SaveFileDialog = Microsoft.Win32.SaveFileDialog;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMRP = Sucrose.Memory.Manage.Readonly.Path;
using SMMVL = Sucrose.Memory.Manage.Valuable.Log;
using SMR = Sucrose.Memory.Readonly;
using SRER = Sucrose.Resources.Extension.Resources;
using SSZEZ = Sucrose.Shared.Zip.Extension.Zip;

namespace Sucrose.Shared.Launcher.Helper
{
    internal static class Create
    {
        public static void Start()
        {
            SaveFileDialog SaveDialog = new()
            {
                FileName = SMMVL.FileNameCompress,

                Filter = SRER.GetValue("Launcher", "SaveDialogFilter"),
                FilterIndex = 1,

                Title = SRER.GetValue("Launcher", "SaveDialogTitle"),

                InitialDirectory = SMMRP.Desktop
            };

            if (SaveDialog.ShowDialog() == true)
            {
                string Destination = SaveDialog.FileName;

                string[] Sources = new[]
                {
                    Path.Combine(SMMRP.ApplicationData, SMR.AppName, SMR.LogFolder),
                    Path.Combine(SMMRP.ApplicationData, SMR.AppName, SMR.SettingFolder)
                };

                string[] Excludes = new[]
                {
                    SMMI.PrivateSettingManager.SettingFile()
                };

                SSZEZ.Compress(Sources, Excludes, Destination);
            }
        }
    }
}