// Lina Desk brand constants and small shared widgets.
//
// Values follow the approved Lina Desk client design (dark tokens from the
// client proposal). Lina Desk is a human-operated support client: the helper
// asks to connect, the person at this device decides.

import 'package:flutter/material.dart';
import 'package:url_launcher/url_launcher.dart';

class LinaDesk {
  LinaDesk._();

  static const String productName = 'Lina Desk';
  static const String companyName = 'Xcode technologies SRL';
  // Title of the card that asks the person to allow a session.
  static const String helperName = 'Support request';
  // The asker is never named: the helper's own computer name means nothing
  // here and reads like a stranger's.
  static const String someoneWantsToConnect =
      'Someone wants to connect to this device';
  static const String scamWarning =
      'Only allow if you called this helper yourself. Xcode technologies SRL never calls you first.';
  static const String notSharingTitle = 'Not sharing yet';
  static const String startSharingTip =
      'Tap Start sharing when your helper asks for your ID.';
  // Shown before Android's "Display over other apps" page opens, so the jump
  // out of the app is not unexplained.
  static const String overlayPermissionTip =
      'Lina Desk needs this to show the session controls on top of other apps.';
  static const String readIdTip = 'Read this number to your helper.';
  static const String nobodyTip =
      'Nobody can see or control this PC until you press Allow.';
  // Shown when the one-time setup was skipped on first start.
  static const String finishSetupTip =
      'Setup is not finished. Your helper cannot use administrator windows on this PC until it is.';
  static const String collectingLogs = 'Collecting logs...';
  static const String logsSaved =
      'Saved to your Desktop. Send that file to your helper.';
  static const String logsFailed = 'The logs could not be collected.';
  static const String sourceUrl =
      'https://github.com/ablake4427/lina-desk-client';

  // Approved dark tokens.
  static const Color bg = Color(0xFF1B1817);
  static const Color bgRail = Color(0xFF221E1C);
  static const Color panel = Color(0xFF2A2523);
  static const Color panel2 = Color(0xFF302A27);
  static const Color line = Color(0xFF3E3633);
  static const Color text = Color(0xFFF5EFEA);
  static const Color text2 = Color(0xFFD0C6BE);
  static const Color muted = Color(0xFFA69B92);
  static const Color accent = Color(0xFF7B4CF5);
  static const Color accentHi = Color(0xFFB9A0FF);
  static const Color accentSoft = Color(0xFF30264A);
  static const Color ok = Color(0xFF5BD69A);
  static const Color okSoft = Color(0xFF25362C);
  static const Color warn = Color(0xFFF2B24C);
  static const Color warnSoft = Color(0xFF3D311F);
  static const Color err = Color(0xFFFF7A72);
  static const Color errSoft = Color(0xFF3F2826);

  static const LinearGradient signature = LinearGradient(
    begin: Alignment.bottomLeft,
    end: Alignment.topRight,
    colors: [Color(0xFF7B4CF5), Color(0xFFC42AA8)],
  );

  static Future<void> openSource() async {
    await launchUrl(Uri.parse(sourceUrl),
        mode: LaunchMode.externalApplication);
  }
}

/// Gradient tile with the monitor glyph (the Lina Desk mark).
class LinaMark extends StatelessWidget {
  final double size;
  const LinaMark({Key? key, this.size = 24}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    // The approved app icon (dark tile, corner brackets, spark).
    return Image.asset('assets/icon.png',
        width: size, height: size, filterQuality: FilterQuality.medium);
  }
}

/// Amber scam warning shown on the home screen and on every request.
class LinaWarning extends StatelessWidget {
  final String text;
  const LinaWarning({Key? key, this.text = LinaDesk.scamWarning})
      : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
      decoration: BoxDecoration(
        color: LinaDesk.warnSoft,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: LinaDesk.warn.withOpacity(0.35)),
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Icon(Icons.shield_outlined, size: 18, color: LinaDesk.warn),
          const SizedBox(width: 10),
          Expanded(
            child: Text(
              text,
              style: const TextStyle(
                  color: LinaDesk.warn, fontSize: 13, height: 1.35),
            ),
          ),
        ],
      ),
    );
  }
}

/// Quiet note ("Nobody can see or control this PC until you press Allow.").
class LinaNote extends StatelessWidget {
  final String text;
  final IconData icon;
  const LinaNote({Key? key, required this.text, this.icon = Icons.lock_outline})
      : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
      decoration: BoxDecoration(
        color: LinaDesk.panel,
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Icon(icon, size: 16, color: LinaDesk.text2),
          const SizedBox(width: 10),
          Expanded(
            child: Text(
              text,
              style: const TextStyle(
                  color: LinaDesk.text2, fontSize: 13, height: 1.35),
            ),
          ),
        ],
      ),
    );
  }
}

/// Company name and the source-code link (the AGPL source offer).
class LinaFooter extends StatelessWidget {
  final Widget? trailing;
  const LinaFooter({Key? key, this.trailing}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.center,
      children: [
        const Expanded(
          child: Text(
            LinaDesk.companyName,
            style: TextStyle(color: LinaDesk.text2, fontSize: 13),
          ),
        ),
        if (trailing != null) trailing!,
        if (trailing != null) const SizedBox(width: 14),
        InkWell(
          onTap: LinaDesk.openSource,
          child: const Text(
            'Source code',
            style: TextStyle(
              color: LinaDesk.accentHi,
              fontSize: 12,
              decoration: TextDecoration.underline,
              decorationColor: LinaDesk.accentHi,
            ),
          ),
        ),
      ],
    );
  }
}
