using System.IO;
using System.Windows.Media.Imaging;
using SEVAMI = Sucrose.Engine.VA.Manage.Internal;

namespace Sucrose.Engine.VA.Event
{
    internal static class Gif
    {
        public static async void ImageTimer_Tick(object sender, EventArgs e)
        {
            SEVAMI.ImageTimer.Stop();

            if (SEVAMI.ImageLoop || SEVAMI.ImageFirst)
            {
                SEVAMI.ImageFirst = false;

                foreach (string Image in SEVAMI.ImageResult.List)
                {
                    while (!SEVAMI.ImageState)
                    {
                        await Task.Delay(1000);
                    }

                    SetImage(Image);

                    string[] Split = Path.GetFileNameWithoutExtension(Image).Split('_');

                    //Thread.Sleep(Convert.ToInt32(Split.Last()));
                    await Task.Delay(Convert.ToInt32(Split.Last()));

                    //Thread.Sleep(Convert.ToInt32(Result.Total / Result.List.Count));
                    //await Task.Delay(Convert.ToInt32(Result.Total / Result.List.Count));
                }
            }

            SEVAMI.ImageTimer.Start();
        }

        public static void SetImage(string Path)
        {
            Uri Uri = new(Path);

            BitmapImage Image = new(Uri);

            SEVAMI.ImageEngine.Source = Image;
        }
    }
}