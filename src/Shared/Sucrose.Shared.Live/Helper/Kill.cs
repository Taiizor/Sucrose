using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSSHL = Sucrose.Shared.Space.Helper.Live;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SWUD = Skylark.Wing.Utility.Desktop;

namespace Sucrose.Shared.Live.Helper
{
    internal static class Kill
    {
        public static void Stop()
        {
            SSSHL.Kill();

            if (!string.IsNullOrEmpty(SMMM.App))
            {
                SSSHP.Kill(SMMM.App);
            }

            SWUD.RefreshDesktop();

            SMMI.AuroraSettingManager.SetSetting(SMC.App, string.Empty);
        }
    }
}