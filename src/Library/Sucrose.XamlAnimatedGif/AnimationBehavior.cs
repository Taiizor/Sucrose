using Sucrose.XamlAnimatedGif.Decoding;
using Sucrose.XamlAnimatedGif.Extensions;
using System.ComponentModel;
using System.IO;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Markup;
using System.Windows.Media.Animation;
using System.Windows.Media.Imaging;

namespace Sucrose.XamlAnimatedGif
{
    public static class AnimationBehavior
    {
        #region Public attached properties and events

        #region SourceUri

        [AttachedPropertyBrowsableForType(typeof(Image))]
        public static Uri GetSourceUri(Image image)
        {
            return (Uri)image.GetValue(SourceUriProperty);
        }

        public static void SetSourceUri(Image image, Uri value)
        {
            image.SetValue(SourceUriProperty, value);
        }

        public static readonly DependencyProperty SourceUriProperty =
            DependencyProperty.RegisterAttached(
              "SourceUri",
              typeof(Uri),
              typeof(AnimationBehavior),
              new PropertyMetadata(
                null,
                SourceChanged));

        #endregion

        #region SourceStream

        [AttachedPropertyBrowsableForType(typeof(Image))]
        [DesignerSerializationVisibility(DesignerSerializationVisibility.Hidden)]
        public static Stream GetSourceStream(DependencyObject obj)
        {
            return (Stream)obj.GetValue(SourceStreamProperty);
        }

        public static void SetSourceStream(DependencyObject obj, Stream value)
        {
            obj.SetValue(SourceStreamProperty, value);
        }

        public static readonly DependencyProperty SourceStreamProperty =
            DependencyProperty.RegisterAttached(
                "SourceStream",
                typeof(Stream),
                typeof(AnimationBehavior),
                new PropertyMetadata(
                    null,
                    SourceChanged));

        #endregion

        #region RepeatBehavior

        [AttachedPropertyBrowsableForType(typeof(Image))]
        public static RepeatBehavior GetRepeatBehavior(DependencyObject obj)
        {
            return (RepeatBehavior)obj.GetValue(RepeatBehaviorProperty);
        }

        public static void SetRepeatBehavior(DependencyObject obj, RepeatBehavior value)
        {
            obj.SetValue(RepeatBehaviorProperty, value);
        }

        public static readonly DependencyProperty RepeatBehaviorProperty =
            DependencyProperty.RegisterAttached(
              "RepeatBehavior",
              typeof(RepeatBehavior),
              typeof(AnimationBehavior),
              new PropertyMetadata(
                default(RepeatBehavior),
                RepeatBehaviorChanged));

        #endregion

        #region CacheFramesInMemory

        public static void SetCacheFramesInMemory(DependencyObject element, bool value)
        {
            element.SetValue(CacheFramesInMemoryProperty, value);
        }

        [AttachedPropertyBrowsableForType(typeof(Image))]
        public static bool GetCacheFramesInMemory(DependencyObject element)
        {
            return (bool)element.GetValue(CacheFramesInMemoryProperty);
        }

        public static readonly DependencyProperty CacheFramesInMemoryProperty =
            DependencyProperty.RegisterAttached(
            "CacheFramesInMemory",
            typeof(bool),
            typeof(AnimationBehavior),
            new PropertyMetadata(false, SourceChanged));

        #endregion

        #region AutoStart

        [AttachedPropertyBrowsableForType(typeof(Image))]
        public static bool GetAutoStart(DependencyObject obj)
        {
            return (bool)obj.GetValue(AutoStartProperty);
        }

        public static void SetAutoStart(DependencyObject obj, bool value)
        {
            obj.SetValue(AutoStartProperty, value);
        }

        public static readonly DependencyProperty AutoStartProperty =
            DependencyProperty.RegisterAttached(
                "AutoStart",
                typeof(bool),
                typeof(AnimationBehavior),
                new PropertyMetadata(true));

        #endregion

        #region AnimateInDesignMode


        public static bool GetAnimateInDesignMode(DependencyObject obj)
        {
            return (bool)obj.GetValue(AnimateInDesignModeProperty);
        }

        public static void SetAnimateInDesignMode(DependencyObject obj, bool value)
        {
            obj.SetValue(AnimateInDesignModeProperty, value);
        }

        public static readonly DependencyProperty AnimateInDesignModeProperty =
            DependencyProperty.RegisterAttached(
                "AnimateInDesignMode",
                typeof(bool),
                typeof(AnimationBehavior),
                new PropertyMetadata(
                    false,
                    AnimateInDesignModeChanged));

        #endregion

        #region ClientUserAgent

        public static string GetClientUserAgent()
        {
            return UriLoader.ClientUserAgent;
        }

        public static void SetClientUserAgent(string userAgent)
        {
            UriLoader.ClientUserAgent = userAgent;
        }

        #endregion

        #region DownloadCacheLocation

        public static string GetDownloadCacheLocation()
        {
            return UriLoader.DownloadCacheLocation;
        }

        public static void ClearDownloadCacheLocation()
        {
            string cacheLocation = UriLoader.DownloadCacheLocation;
            if (Directory.Exists(cacheLocation))
            {
                foreach (string cacheFile in Directory.GetFiles(cacheLocation))
                {
                    try
                    {
                        File.Delete(cacheFile);
                    }
                    catch { }
                }
            }
        }

        public static void SetDownloadCacheLocation(string value)
        {
            UriLoader.DownloadCacheLocation = value;
        }

        #endregion

        #region DownloadCacheExpiration

        public static TimeSpan GetDownloadCacheExpiration()
        {
            return UriLoader.DownloadCacheExpiration;
        }

        public static void SetDownloadCacheExpiration(TimeSpan duration)
        {
            UriLoader.DownloadCacheExpiration = duration;
        }

        #endregion

        #region Animator

        [DesignerSerializationVisibility(DesignerSerializationVisibility.Hidden)]
        public static Animator GetAnimator(DependencyObject obj)
        {
            return (Animator)obj.GetValue(AnimatorProperty);
        }

        private static void SetAnimator(DependencyObject obj, Animator value)
        {
            obj.SetValue(AnimatorProperty, value);
        }

        public static readonly DependencyProperty AnimatorProperty =
            DependencyProperty.RegisterAttached(
                "Animator",
                typeof(Animator),
                typeof(AnimationBehavior),
                new PropertyMetadata(null));

        #endregion

        #region Error

        public static readonly RoutedEvent ErrorEvent =
            EventManager.RegisterRoutedEvent(
                "Error",
                RoutingStrategy.Bubble,
                typeof(AnimationErrorEventHandler),
                typeof(AnimationBehavior));

        public static void AddErrorHandler(DependencyObject d, AnimationErrorEventHandler handler)
        {
            (d as UIElement)?.AddHandler(ErrorEvent, handler);
        }

        public static void RemoveErrorHandler(DependencyObject d, AnimationErrorEventHandler handler)
        {
            (d as UIElement)?.RemoveHandler(ErrorEvent, handler);
        }

        internal static void OnError(Image image, Exception exception, AnimationErrorKind kind)
        {
            image.RaiseEvent(new AnimationErrorEventArgs(image, exception, kind));
        }

        private static void AnimatorError(object sender, AnimationErrorEventArgs e)
        {
            UIElement source = e.Source as UIElement;
            source?.RaiseEvent(e);
        }

        #endregion

        #region DownloadProgress

        public static readonly RoutedEvent DownloadProgressEvent =
            EventManager.RegisterRoutedEvent(
                "DownloadProgress",
                RoutingStrategy.Bubble,
                typeof(DownloadProgressEventHandler),
                typeof(AnimationBehavior));

        public static void AddDownloadProgressHandler(DependencyObject d, DownloadProgressEventHandler handler)
        {
            (d as UIElement)?.AddHandler(DownloadProgressEvent, handler);
        }

        public static void RemoveDownloadProgressHandler(DependencyObject d, DownloadProgressEventHandler handler)
        {
            (d as UIElement)?.RemoveHandler(DownloadProgressEvent, handler);
        }

        internal static void OnDownloadProgress(Image image, int downloadPercentage)
        {
            image.RaiseEvent(new DownloadProgressEventArgs(image, downloadPercentage));
        }
        #endregion

        #region Loaded

        public static readonly RoutedEvent LoadedEvent =
            EventManager.RegisterRoutedEvent(
                "Loaded",
                RoutingStrategy.Bubble,
                typeof(RoutedEventHandler),
                typeof(AnimationBehavior));

        public static void AddLoadedHandler(DependencyObject d, RoutedEventHandler handler)
        {
            (d as UIElement)?.AddHandler(LoadedEvent, handler);
        }

        public static void RemoveLoadedHandler(DependencyObject d, RoutedEventHandler handler)
        {
            (d as UIElement)?.RemoveHandler(LoadedEvent, handler);
        }

        private static void OnLoaded(Image sender)
        {
            sender.RaiseEvent(new RoutedEventArgs(LoadedEvent, sender));
        }

        #endregion

        #region AnimationStarted

        public static readonly RoutedEvent AnimationStartedEvent =
            EventManager.RegisterRoutedEvent(
                "AnimationStarted",
                RoutingStrategy.Bubble,
                typeof(AnimationStartedEventHandler),
                typeof(AnimationBehavior));

        public static void AddAnimationStartedHandler(DependencyObject d, AnimationStartedEventHandler handler)
        {
            (d as UIElement)?.AddHandler(AnimationStartedEvent, handler);
        }

        public static void RemoveAnimationStartedHandler(DependencyObject d, AnimationStartedEventHandler handler)
        {
            (d as UIElement)?.RemoveHandler(AnimationStartedEvent, handler);
        }

        private static void AnimatorAnimationStarted(object sender, AnimationStartedEventArgs e)
        {
            (e.Source as Image)?.RaiseEvent(e);
        }

        #endregion

        #region AnimationCompleted

        public static readonly RoutedEvent AnimationCompletedEvent =
            EventManager.RegisterRoutedEvent(
                "AnimationCompleted",
                RoutingStrategy.Bubble,
                typeof(AnimationCompletedEventHandler),
                typeof(AnimationBehavior));

        public static void AddAnimationCompletedHandler(DependencyObject d, AnimationCompletedEventHandler handler)
        {
            (d as UIElement)?.AddHandler(AnimationCompletedEvent, handler);
        }

        public static void RemoveAnimationCompletedHandler(DependencyObject d, AnimationCompletedEventHandler handler)
        {
            (d as UIElement)?.RemoveHandler(AnimationCompletedEvent, handler);
        }

        private static void AnimatorAnimationCompleted(object sender, AnimationCompletedEventArgs e)
        {
            (e.Source as Image)?.RaiseEvent(e);
        }

        #endregion

        #endregion

        #region Private attached properties

        private static int GetSeqNum(DependencyObject obj)
        {
            return (int)obj.GetValue(SeqNumProperty);
        }

        private static void SetSeqNum(DependencyObject obj, int value)
        {
            obj.SetValue(SeqNumProperty, value);
        }

        private static readonly DependencyProperty SeqNumProperty =
            DependencyProperty.RegisterAttached("SeqNum", typeof(int), typeof(AnimationBehavior), new PropertyMetadata(0));

        #endregion

        private static void SourceChanged(DependencyObject o, DependencyPropertyChangedEventArgs e)
        {
            if (o is not Image image)
            {
                return;
            }

            InitAnimation(image);
        }

        private static void RepeatBehaviorChanged(DependencyObject o, DependencyPropertyChangedEventArgs e)
        {
            GetAnimator(o)?.OnRepeatBehaviorChanged();
        }

        private static void AnimateInDesignModeChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
        {
            if (d is not Image image)
            {
                return;
            }

            InitAnimation(image);
        }

        private static bool CheckDesignMode(Image image, Uri sourceUri, Stream sourceStream)
        {
            if (IsInDesignMode(image) && !GetAnimateInDesignMode(image))
            {
                try
                {
                    if (sourceStream != null)
                    {
                        SetStaticImage(image, sourceStream);
                    }
                    else if (sourceUri != null)
                    {
                        BitmapImage bmp = new()
                        {
                            UriSource = sourceUri
                        };
                        image.Source = bmp;
                    }
                }
                catch
                {
                    image.Source = null;
                }
                return false;
            }
            return true;
        }

        private static void InitAnimation(Image image)
        {
            if (IsLoaded(image))
            {
                image.Unloaded += Image_Unloaded;
            }
            else
            {
                image.Loaded += Image_Loaded;
                return;
            }

            int seqNum = GetSeqNum(image) + 1;
            SetSeqNum(image, seqNum);

            image.Source = null;
            ClearAnimatorCore(image);

            try
            {
                Stream stream = GetSourceStream(image);
                if (stream != null)
                {
                    InitAnimationAsync(image, stream.AsBuffered(), GetRepeatBehavior(image), seqNum, GetCacheFramesInMemory(image));
                    return;
                }

                Uri uri = GetAbsoluteUri(image);
                if (uri != null)
                {
                    InitAnimationAsync(image, uri, GetRepeatBehavior(image), seqNum, GetCacheFramesInMemory(image));
                }
            }
            catch (Exception ex)
            {
                OnError(image, ex, AnimationErrorKind.Loading);
            }
        }

        private static void Image_Loaded(object sender, RoutedEventArgs e)
        {
            Image image = (Image)sender;
            image.Loaded -= Image_Loaded;
            InitAnimation(image);
        }

        private static void Image_Unloaded(object sender, RoutedEventArgs e)
        {
            Image image = (Image)sender;
            image.Unloaded -= Image_Unloaded;
            image.Loaded += Image_Loaded;

            int seqNum = GetSeqNum(image) + 1;
            SetSeqNum(image, seqNum);

            image.Source = null;
            ClearAnimatorCore(image);
        }

        private static bool IsLoaded(FrameworkElement element)
        {
            return element.IsLoaded;
        }

        private static Uri GetAbsoluteUri(Image image)
        {
            Uri uri = GetSourceUri(image);
            if (uri == null)
            {
                return null;
            }

            if (!uri.IsAbsoluteUri)
            {
                Uri baseUri = ((IUriContext)image).BaseUri;
                if (baseUri != null)
                {
                    uri = new Uri(baseUri, uri);
                }
                else
                {
                    throw new InvalidOperationException("Relative URI can't be resolved");
                }
            }
            return uri;
        }

        private static async void InitAnimationAsync(Image image, Uri sourceUri, RepeatBehavior repeatBehavior, int seqNum, bool cacheFrameDataInMemory)
        {
            if (!CheckDesignMode(image, sourceUri, null))
            {
                return;
            }

            try
            {
                Progress<int> progress = new(percentage => OnDownloadProgress(image, percentage));
                ImageAnimator animator = await ImageAnimator.CreateAsync(sourceUri, repeatBehavior, progress, image, cacheFrameDataInMemory);
                // Check that the source hasn't changed while we were loading the animation
                if (GetSeqNum(image) != seqNum)
                {
                    animator.Dispose();
                    return;
                }

                SetAnimatorCore(image, animator);
                OnLoaded(image);
                await StartAsync(image, animator);
            }
            catch (InvalidSignatureException)
            {
                await SetStaticImageAsync(image, sourceUri);
                OnLoaded(image);
            }
            catch (Exception ex)
            {
                OnError(image, ex, AnimationErrorKind.Loading);
            }
        }

        private static async void InitAnimationAsync(Image image, Stream stream, RepeatBehavior repeatBehavior, int seqNum, bool cacheFrameDataInMemory)
        {
            if (!CheckDesignMode(image, null, stream))
            {
                return;
            }

            try
            {
                ImageAnimator animator = await ImageAnimator.CreateAsync(stream, repeatBehavior, image, cacheFrameDataInMemory);
                // Check that the source hasn't changed while we were loading the animation
                if (GetSeqNum(image) != seqNum)
                {
                    animator.Dispose();
                    return;
                }

                SetAnimatorCore(image, animator);
                OnLoaded(image);
                await StartAsync(image, animator);
            }
            catch (InvalidSignatureException)
            {
                SetStaticImage(image, stream);
                OnLoaded(image);
            }
            catch (Exception ex)
            {
                OnError(image, ex, AnimationErrorKind.Loading);
            }
        }

        private static void SetAnimatorCore(Image image, Animator animator)
        {
            SetAnimator(image, animator);
            animator.Error += AnimatorError;
            animator.AnimationStarted += AnimatorAnimationStarted;
            animator.AnimationCompleted += AnimatorAnimationCompleted;
            image.Source = animator.Bitmap;
        }

        private static async Task StartAsync(Image image, Animator animator)
        {
            if (GetAutoStart(image))
            {
                animator.Play();
            }
            else
            {
                await animator.ShowFirstFrameAsync();
            }
        }

        private static void ClearAnimatorCore(Image image)
        {
            Animator animator = GetAnimator(image);
            if (animator == null)
            {
                return;
            }

            animator.AnimationCompleted -= AnimatorAnimationCompleted;
            animator.AnimationStarted -= AnimatorAnimationStarted;
            animator.Error -= AnimatorError;
            animator.Dispose();
            SetAnimator(image, null);
        }

        // ReSharper disable once UnusedParameter.Local (used in WPF)
        private static bool IsInDesignMode(DependencyObject obj)
        {
            return DesignerProperties.GetIsInDesignMode(obj);
        }

        private static async Task SetStaticImageAsync(Image image, Uri sourceUri)
        {
            try
            {
                Progress<int> progress = new(percentage => OnDownloadProgress(image, percentage));
                using Stream stream = await UriLoader.GetStreamFromUriAsync(sourceUri, progress);
                SetStaticImageCore(image, stream);
            }
            catch (Exception ex)
            {
                OnError(image, ex, AnimationErrorKind.Loading);
            }
        }

        private static void SetStaticImage(Image image, Stream stream)
        {
            try
            {
                SetStaticImageCore(image, stream);
            }
            catch (Exception ex)
            {
                OnError(image, ex, AnimationErrorKind.Loading);
            }
        }

        private static void SetStaticImageCore(Image image, Stream stream)
        {
            stream.Seek(0, SeekOrigin.Begin);
            BitmapImage bmp = new();
            bmp.BeginInit();
            bmp.CacheOption = BitmapCacheOption.OnLoad;
            bmp.StreamSource = stream;
            bmp.EndInit();
            image.Source = bmp;
        }
    }
}