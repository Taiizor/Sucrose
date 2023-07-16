using SEAAMI = Sucrose.Engine.AA.Manage.Internal;

namespace Sucrose.Engine.AA.Helper
{
    internal static class Ready
    {
        public static bool Check()
        {
            return SEAAMI.ApplicationHandle == IntPtr.Zero || SEAAMI.ApplicationProcess == null;
        }
    }
}