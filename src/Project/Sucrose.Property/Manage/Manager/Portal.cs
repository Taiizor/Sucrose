using Wpf.Ui.Controls;
using SMMCP = Sucrose.Memory.Manage.Constant.Portal;
using SMMI = Sucrose.Manager.Manage.Internal;
using SPMI = Sucrose.Property.Manage.Internal;

namespace Sucrose.Property.Manage.Manager
{
    internal static class Portal
    {
        public static WindowBackdropType BackdropType => SMMI.PortalSettingManager.GetSetting(SMMCP.BackdropType, SPMI.DefaultBackdropType);
    }
}