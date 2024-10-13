using SMMCL = Sucrose.Memory.Manage.Constant.Library;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMRF = Sucrose.Memory.Manage.Readonly.Folder;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;
using SMMRP = Sucrose.Memory.Manage.Readonly.Path;

namespace Sucrose.Manager.Manage
{
    public static class Library
    {
        public static string Location => SMMI.LibrarySettingManager.GetSetting(SMMCL.Location, Path.Combine(SMMRP.ApplicationData, SMMRG.AppName, SMMRF.Library));

        public static bool DeleteCorrupt => SMMI.LibrarySettingManager.GetSetting(SMMCL.DeleteCorrupt, false);

        public static string Selected => SMMI.LibrarySettingManager.GetSetting(SMMCL.Selected, string.Empty);

        public static bool DeleteConfirm => SMMI.LibrarySettingManager.GetSetting(SMMCL.DeleteConfirm, true);

        public static bool Move => SMMI.LibrarySettingManager.GetSetting(SMMCL.Move, true);
    }
}