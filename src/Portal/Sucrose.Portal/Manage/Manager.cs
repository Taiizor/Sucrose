using System.Net;
using System.Windows.Media;
using Wpf.Ui.Controls;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SWHWT = Skylark.Wing.Helper.WindowsTheme;

namespace Sucrose.Portal.Manage
{
    internal static class Manager
    {
        public static Stretch BackgroundStretch => SMMI.PortalSettingManager.GetSetting(SMC.BackgroundStretch, DefaultBackgroundStretch);

        public static WindowBackdropType BackdropType => SMMI.PortalSettingManager.GetSetting(SMC.BackdropType, DefaultBackdropType);

        public static IPAddress Host => SMMI.LauncherSettingManager.GetSettingAddress(SMC.Host, IPAddress.Loopback);

        public static SEWTT Theme => SMMI.GeneralSettingManager.GetSetting(SMC.ThemeType, SWHWT.GetTheme());

        public static WindowBackdropType DefaultBackdropType => WindowBackdropType.None;

        public static Stretch DefaultBackgroundStretch => Stretch.UniformToFill;

        public static Mutex Mutex => new(true, SMR.PortalMutex);
    }
}