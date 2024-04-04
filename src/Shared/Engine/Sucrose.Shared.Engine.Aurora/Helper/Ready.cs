using SSEAMI = Sucrose.Shared.Engine.Aurora.Manage.Internal;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;

namespace Sucrose.Shared.Engine.Aurora.Helper
{
    internal static class Ready
    {
        public static bool Check(int Count)
        {
            return !SSEMI.Applications.Any() && SSEMI.Applications.Count < Count && SSSHP.WorkCount(SSEAMI.Application) < Count;
        }
    }
}