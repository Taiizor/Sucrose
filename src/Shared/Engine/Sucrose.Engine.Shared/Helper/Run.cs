using SSDEET = Sucrose.Shared.Dependency.Enum.EngineType;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;

namespace Sucrose.Engine.Shared.Helper
{
    internal static class Run
    {
        public static bool Check()
        {
            int Result = 0;

            foreach (KeyValuePair<SSDEET, string> Pair in SSSMI.EngineLive)
            {
                if (SSSHP.Work(Pair.Value))
                {
                    Result += SSSHP.WorkCount(SSSMI.EngineLive[Pair.Key]);
                }
            }

            return Result <= 1;
        }
    }
}