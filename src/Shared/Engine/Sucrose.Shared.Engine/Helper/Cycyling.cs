using SMMC = Sucrose.Manager.Manage.Cycling;
using SMMCC = Sucrose.Memory.Manage.Constant.Cycling;
using SMMI = Sucrose.Manager.Manage.Internal;
using SSSHC = Sucrose.Shared.Space.Helper.Cycyling;
using Timer = System.Timers.Timer;

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
                SMMI.CyclingSettingManager.SetSetting(SMMCC.PassingCycyling, SMMC.PassingCycyling + Second);

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