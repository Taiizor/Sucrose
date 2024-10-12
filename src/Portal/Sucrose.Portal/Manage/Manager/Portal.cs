using Wpf.Ui.Controls;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMCP = Sucrose.Memory.Manage.Constant.Portal;
using SPMI = Sucrose.Portal.Manage.Internal;

namespace Sucrose.Portal.Manage.Manager
{
    internal static class Portal
    {
        public static WindowBackdropType BackdropType => SMMI.PortalSettingManager.GetSetting(SMMCP.BackdropType, SPMI.DefaultBackdropType);
    }
}