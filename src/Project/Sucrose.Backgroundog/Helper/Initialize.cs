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
            TimerCallback Callback = InitializeTimer_Callback;
            SBMI.InitializeTimer = new(Callback, null, 0, 1000);
        }

        public static void Stop()
        {
            SBMI.InitializeTimer.Dispose();
        }

        private static async void InitializeTimer_Callback(object State)
        {
            if (SBMI.Processing)
            {
                SBMI.Processing = false;
                Console.WriteLine("Callback");

                //Libredeki cpu, ram, net bilgileri değişkene atılacak
                SBHC.Start();

                if (SSSHL.Run())
                {
                    Console.WriteLine("Run");
                    //Performans şartları kontrol edilecek...
                    SBHP.Start();
                }
                else if (SBMI.Condition)
                {
                    Console.WriteLine("Condition");
                    //Performans şartları sağlanıp live motoru kapatıldıysa veya durdurulduysa
                    //Live motoruna gRPC (Live.json) ile durdur mesajı gönderilir  
                }
                else
                {
                    Console.WriteLine("Deneme");
                    int MaxAttempts = 5;
                    bool Success = false;
                    int IntervalInSeconds = 1;

                    for (int Attempt = 0; Attempt <= MaxAttempts; Attempt++)
                    {
                        Success = SSSHL.Run();

                        if (Success)
                        {
                            Console.WriteLine("İşlem başarılı!");
                            break;
                        }

                        await Task.Delay(IntervalInSeconds * 1000);
                    }

                    if (!Success)
                    {
                        Console.WriteLine("İşlem başarısız oldu ve 5 deneme sonunda hala true dönmedi.");
                        SBMI.Exit = false;
                        Stop();
                    }
                }

                SBMI.Processing = true;
            }
        }
    }
}
