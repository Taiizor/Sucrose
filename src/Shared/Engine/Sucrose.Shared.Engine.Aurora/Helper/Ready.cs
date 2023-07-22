using SSEAMI = Sucrose.Shared.Engine.Aurora.Manage.Internal;

namespace Sucrose.Shared.Engine.Aurora.Helper
{
    internal static class Ready
    {
        public static bool Check()
        {
            return SSEAMI.ApplicationHandle == IntPtr.Zero || SSEAMI.ApplicationProcess == null;
        }
    }
}