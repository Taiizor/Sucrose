using SEAAMI = Sucrose.Engine.AA.Manage.Internal;
using SSHP = Sucrose.Space.Helper.Processor;

namespace Sucrose.Engine.AA.Helper
{
    internal static class Ready
    {
        public static bool Check()
        {
            return SEAAMI.ApplicationHandle == IntPtr.Zero || SEAAMI.ApplicationProcess == null || !SSHP.Work(SEAAMI.Application);
        }
    }
}