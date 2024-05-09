using SMMM = Sucrose.Manager.Manage.Manager;
using SMMI = Sucrose.Manager.Manage.Internal;
using SSSHC = Sucrose.Shared.Space.Helper.Cycyling;
using Timer = System.Timers.Timer;
using SMC = Sucrose.Memory.Constant;

namespace Sucrose.Shared.Engine.Helper
{
    internal static class Cycyling
    {
        public static void Start()
        {
            int Second = 10;

            Timer Cycler = new(Second * 1000);

            Cycler.Elapsed += (s, e) =>
            {
                SMMI.CyclingManager.SetSetting(SMC.PassingCycyling, SMMM.PassingCycyling + Second);

                if (SSSHC.Check())
                {
                    SSSHC.Change();
                }
            };

            Cycler.AutoReset = true;

            Cycler.Start();
        }
    }
}