using SBMI = Sucrose.Backgroundog.Manage.Internal;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSSHL = Sucrose.Shared.Space.Helper.Live;
using SBHC = Sucrose.Backgroundog.Helper.Counter;
using SBHP = Sucrose.Backgroundog.Helper.Performance;

namespace Sucrose.Backgroundog.Helper
{
    internal static class Initialize
    {
        public static void Start()
        {
            SBMI.InitializeTimer.Tick += new EventHandler(InitializeTimer_Tick);
            SBMI.InitializeTimer.Interval = new TimeSpan(0, 0, 1);
            SBMI.InitializeTimer.Start();
        }

        public static void Stop()
        {
            SBMI.InitializeTimer.Stop();
        }

        private static void InitializeTimer_Tick(object sender, EventArgs e)
        {
            //Libredeki cpu, ram, net bilgileri değişkene atılacak
            SBHC.Start();

            if (SSSHL.Run())
            {
                //Performans şartları kontrol edilecek...
                SBHP.Start();
            }
            else if (SBMI.Condition)
            {
                //Performans şartları sağlanıp live motoru kapatıldıysa veya durdurulduysa
                //Live motoruna gRPC (Live.json) ile durdur mesajı gönderilir  
            }
        }
    }
}
