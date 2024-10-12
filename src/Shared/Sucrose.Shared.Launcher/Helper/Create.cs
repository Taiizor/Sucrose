using System.IO;
using SaveFileDialog = Microsoft.Win32.SaveFileDialog;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMRP = Sucrose.Memory.Manage.Readonly.Path;
using SMMVL = Sucrose.Memory.Manage.Valuable.Log;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;
using SMR = Sucrose.Memory.Readonly;
using SRER = Sucrose.Resources.Extension.Resources;
using SMMRF = Sucrose.Memory.Manage.Readonly.Folder;
using SMMRC = Sucrose.Memory.Manage.Readonly.Content;
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
                    Path.Combine(SMMRP.ApplicationData, SMMRG.AppName, SMMRF.Log),
                    Path.Combine(SMMRP.ApplicationData, SMMRG.AppName, SMMRF.Setting)
                };

                string[] Excludes = new[]
                {
                    SMMI.ObjectionableSettingManager.SettingFile()
                };

                SSZEZ.Compress(Sources, Excludes, Destination);
            }
        }
    }
}